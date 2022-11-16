//! Host-dispatched task finder.

use crate::{Host, InnerHostConfig, InnerGlobalStatus, InnerAPI, TaskMapTaskInfo, HostError};
use crate::functional::finder;
use std::{collections, sync::Arc};
use futures::{prelude::*, channel::{mpsc, oneshot}, future::FusedFuture};
use tokio::{time, sync, task};
use tracing::{event, Level};

#[derive(Debug)]
pub(crate) enum FinderCommand {
    /// stop and shutdown task finder.
    Shutdown {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// abort a running task finder process, returning to sleep.
    AbortRunning {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// wake up and let it run immediately.
    RunNow {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// query status
    QueryStatus {
        receipt: oneshot::Sender<Result<FinderStatus, HostError>>,
    }
}

impl Host {
    /// the task finder routine being initialized.
    async fn task_finder_rt(
        host_config: Arc<InnerHostConfig>,
        global_status: Arc<InnerGlobalStatus>,
        task_map: Arc<sync::RwLock<collections::HashMap<u32, TaskMapTaskInfo>>>,
        inner_api: Arc<InnerAPI>,
        mut listener: mpsc::UnboundedReceiver<FinderCommand>)
    {
        const TASK_FINDER_CRON: &str = "0 0,10,20,30,40,50 * * * * *"; // every day every hour at 0,10,20,30,40,50-min.
        let mut shutdown_announce_handle = None;
        let mut last_run_result = FinderRoutineStatus::NoRun;
        let mut last_run_time = chrono::DateTime::<chrono::Utc>::MIN_UTC;

        let sleep = time::sleep(time::Duration::from_nanos(0)).fuse(); // the first sleep would and should expire immediately
        let mut sleep = Box::pin(sleep);
        let find_exec = future::Fuse::terminated();
        let mut find_exec = Box::pin(find_exec);

        // main message loop
        loop {
            futures::select! {
                _ = sleep => { // the main query starts here
                    if shutdown_announce_handle.is_none() && find_exec.is_terminated() {
                        let host_config = host_config.clone();
                        let global_status = global_status.clone();
                        let inner_api = inner_api.clone();
                        let task_map = task_map.clone();
                        find_exec.set(async move {
                            let mut task_map = task_map.write().await;
                            finder::FinderExec {
                                host_config,
                                global_status,
                                inner_api,
                                task_map: &mut task_map,
                            }.execute().await
                        }.fuse());
                        last_run_result = FinderRoutineStatus::Running;
                        last_run_time = chrono::Utc::now();
                    }
                    // reset sleep
                    sleep.set(Self::future_by_cron(TASK_FINDER_CRON));
                },
                result = &mut find_exec => { // result from task finding process
                    last_run_result = FinderRoutineStatus::Finished(result);
                    find_exec.set(future::Fuse::terminated());
                },
                cmd = listener.select_next_some() => { // react to command
                    // channel is alive and there are commands.
                    match cmd {
                        FinderCommand::Shutdown { receipt } => {
                            // shutting down
                            if shutdown_announce_handle.is_some() {
                                if receipt.send(Err(HostError::IsShuttingDown)).is_err() {
                                    event!(Level::WARN, "finder command `shutdown` receipt not sent, receiver hung up.");
                                }
                            } else {
                                // set handle
                                shutdown_announce_handle = Some(receipt);
                                // kill running task
                                find_exec.set(future::Fuse::terminated());
                                listener.close();
                                // terminate sleeping future
                                sleep.set(future::Fuse::terminated());
                            }
                        },
                        FinderCommand::AbortRunning { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if !find_exec.is_terminated() {
                                find_exec.set(future::Fuse::terminated());
                                last_run_result = FinderRoutineStatus::Aborted;
                                Ok(())
                            } else {
                                Err(HostError::NotRunning)
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, "finder command `abort` receipt not sent, receiver hung up.");
                            }
                        },
                        FinderCommand::RunNow { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if !find_exec.is_terminated() {
                                Err(HostError::AlreadyRunning)
                            } else {
                                // reset sleep timer such that it expires immediately.
                                sleep.set(time::sleep(time::Duration::from_nanos(0)).fuse());
                                Ok(())
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, "finder command `runnow` receipt not sent, receiver hung up.");
                            }
                        },
                        FinderCommand::QueryStatus { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else {
                                // return details on the task.
                                let frame = FinderStatus {
                                    last_run_status: last_run_result.clone(),
                                    last_run_time,
                                };
                                Ok(frame)
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, "finder command `query` receipt not sent, receiver hung up.");
                            }
                        }
                    }
                },
                complete => break,
            }
        }

        // there should be a shutdown announce handle
        if let Some(announce_handle) = shutdown_announce_handle {
            // send result
            if announce_handle.send(Ok(())).is_err() {
                event!(Level::WARN, "finder command `shutdown` receipt not sent, receiver hung up.");
            }
        } else {
            event!(Level::WARN, "finder routine unexpectedly exited.");
        }
    }

    /// dispatch and start the task finder routine.
    pub(crate) fn start_task_finder(
        host_config: Arc<InnerHostConfig>,
        global_status: Arc<InnerGlobalStatus>,
        task_map: Arc<sync::RwLock<collections::HashMap<u32, TaskMapTaskInfo>>>,
        inner_api: Arc<InnerAPI>,
    ) -> (task::JoinHandle<()>, mpsc::UnboundedSender<FinderCommand>) {
        let (tx, rx) = mpsc::unbounded();
        let handle = tokio::spawn(Self::task_finder_rt(
            host_config,
            global_status,
            task_map,
            inner_api,
            rx));
        (handle, tx)
    }
}

#[derive(Debug, Clone)]
pub struct FinderStatus {
    pub last_run_status: FinderRoutineStatus,
    pub last_run_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum FinderRoutineStatus {
    /// No finder was run yet
    NoRun,
    /// The finder process was running
    Running,
    /// The finder process was forcefully aborted.
    Aborted,
    /// The finder process was finished, with corresponding results.
    Finished(finder::FinderSummary),
}
