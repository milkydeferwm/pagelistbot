//! Host-dispatched API refresher.

use crate::{Host, InnerHostConfig, InnerAPI, HostError};
use crate::functional::refresh;
use std::sync::Arc;
use futures::{prelude::*, channel::{mpsc, oneshot}, future::FusedFuture};
use tokio::{task, time};
use tracing::{event, Level};

#[derive(Debug)]
pub(crate) enum RefresherCommand {
    /// stop and shutdown API refresher.
    Shutdown {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// abort a running API refresher process, returning to sleep.
    AbortRunning {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// wake up and let it run immediately.
    RunNow {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// query status
    QueryStatus {
        receipt: oneshot::Sender<Result<RefresherStatus, HostError>>,
    }
}

impl Host {
    /// the API refresher routine being initialized.
    async fn refresher_rt(
        host_config: Arc<InnerHostConfig>,
        inner_api: Arc<InnerAPI>,
        mut listener: mpsc::UnboundedReceiver<RefresherCommand>)
    {
        const REFRESHER_CRON: &str = "0 0 * * * * *"; // every day every hour at 0 min.
        let mut shutdown_announce_handle = None;
        let mut last_run_result = RefresherRoutineStatus::NoRun;
        let mut last_run_time = chrono::DateTime::<chrono::Utc>::MIN_UTC;

        let sleep = Self::future_by_cron(REFRESHER_CRON); // the first sleep would and should expire immediately
        let mut sleep = Box::pin(sleep);
        let refresh_exec = future::Fuse::terminated();
        let mut refresh_exec = Box::pin(refresh_exec);

        // main message loop
        loop {
            futures::select! {
                _ = sleep => { // the main refresher starts here
                    if shutdown_announce_handle.is_none() && refresh_exec.is_terminated() {
                        let host_config = host_config.clone();
                        let inner_api = inner_api.clone();
                        refresh_exec.set(async move {
                            let mut api = inner_api.api.write().await;
                            let mut siteinfo = inner_api.siteinfo.write().await;
                            let mut has_bot_flag = inner_api.has_bot_flag.write().await;
                            refresh::RefresherExec {
                                host_config,
                                api: &mut api,
                                siteinfo: &mut siteinfo,
                                has_bot_flag: &mut has_bot_flag,
                            }.execute().await
                        }.fuse());
                        last_run_result = RefresherRoutineStatus::Running;
                        last_run_time = chrono::Utc::now();
                    }
                    // reset sleep
                    sleep.set(Self::future_by_cron(REFRESHER_CRON));
                },
                result = &mut refresh_exec => { // result from API refresher process
                    last_run_result = RefresherRoutineStatus::Finished(result);
                    refresh_exec.set(future::Fuse::terminated());
                },
                cmd = listener.select_next_some() => { // react to command
                    // channel is alive and there are commands.
                    match cmd {
                        RefresherCommand::Shutdown { receipt } => {
                            // shutting down
                            if shutdown_announce_handle.is_some() {
                                if receipt.send(Err(HostError::IsShuttingDown)).is_err() {
                                    event!(Level::WARN, "refresher command `shutdown` receipt not sent, receiver hung up.");
                                }
                            } else {
                                // set handle
                                shutdown_announce_handle = Some(receipt);
                                // kill running refresher
                                refresh_exec.set(future::Fuse::terminated());
                                listener.close();
                                // terminate sleeping future
                                sleep.set(future::Fuse::terminated());
                            }
                        },
                        RefresherCommand::AbortRunning { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if !refresh_exec.is_terminated() {
                                refresh_exec.set(future::Fuse::terminated());
                                last_run_result = RefresherRoutineStatus::Aborted;
                                Ok(())
                            } else {
                                Err(HostError::NotRunning)
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, "refresher command `abort` receipt not sent, receiver hung up.");
                            }
                        },
                        RefresherCommand::RunNow { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if !refresh_exec.is_terminated() {
                                Err(HostError::AlreadyRunning)
                            } else {
                                // reset sleep timer such that it expires immediately.
                                sleep.set(time::sleep(time::Duration::from_nanos(0)).fuse());
                                Ok(())
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, "refresher command `runnow` receipt not sent, receiver hung up.");
                            }
                        },
                        RefresherCommand::QueryStatus { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else {
                                // return details on the task.
                                let frame = RefresherStatus {
                                    last_run_status: last_run_result.clone(),
                                    last_run_time,
                                };
                                Ok(frame)
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, "refresher command `query` receipt not sent, receiver hung up.");
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
                event!(Level::WARN, "refresher command `shutdown` receipt not sent, receiver hung up.");
            }
        } else {
            event!(Level::WARN, "refresher routine unexpectedly exited.");
        }
    }

    /// dispatch and start the API refresher routine.
    pub(crate) fn start_api_refresher(
        host_config: Arc<InnerHostConfig>,
        inner_api: Arc<InnerAPI>,
    ) -> (task::JoinHandle<()>, mpsc::UnboundedSender<RefresherCommand>) {
        let (tx, rx) = mpsc::unbounded();
        let handle = tokio::spawn(Self::refresher_rt(
            host_config,
            inner_api,
            rx));
        (handle, tx)
    }
}

#[derive(Debug, Clone)]
pub struct RefresherStatus {
    pub last_run_status: RefresherRoutineStatus,
    pub last_run_time: chrono::DateTime<chrono::Utc>,
}

impl From<RefresherStatus> for interface::types::status::refresher::PageListBotRefresherStatus {
    fn from(value: RefresherStatus) -> Self {
        let RefresherStatus { last_run_status, last_run_time } = value;
        Self {
            last_run_status: last_run_status.into(),
            last_run_time,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RefresherRoutineStatus {
    /// No refresher process was run yet
    NoRun,
    /// The refresher process was running
    Running,
    /// The refresher process was forcefully aborted.
    Aborted,
    /// The refresher process was finished, with corresponding results.
    Finished(refresh::RefresherSummary),
}

impl From<RefresherRoutineStatus> for interface::types::status::refresher::PageListBotRefresherRoutineStatus {
    fn from(value: RefresherRoutineStatus) -> Self {
        match value {
            RefresherRoutineStatus::NoRun => Self::NoRun,
            RefresherRoutineStatus::Running => Self::Running,
            RefresherRoutineStatus::Aborted => Self::Aborted,
            RefresherRoutineStatus::Finished(e) => Self::Finished(e.into()),
        }
    }
}
