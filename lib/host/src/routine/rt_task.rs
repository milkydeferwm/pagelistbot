//! Host-dispatched task scheduler.

use crate::{Host, HostError, InnerHostConfig, InnerGlobalStatus, InnerAPI};
use crate::types;
use crate::functional::task;
use std::sync::Arc;
use futures::{prelude::*, channel::{mpsc, oneshot}, future::FusedFuture};
use tokio::time;
use tracing::{event, Level};

#[derive(Debug)]
pub(crate) enum TaskCommand {
    /// stop and shutdown current task.
    Shutdown {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// update task
    UpdateTaskConfig {
        new_config: types::TaskDescription,
        new_revid: u64,
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// abort a running task, returning to sleep.
    AbortRunning {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// wake up a running task, let it run immediately.
    RunNow {
        receipt: oneshot::Sender<Result<(), HostError>>,
    },
    /// query status
    QueryStatus {
        receipt: oneshot::Sender<Result<TaskStatus, HostError>>,
    }
}

impl Host {
    /// the task routine being dispatched.
    pub(crate) async fn task_rt(
        inner_host_config: Arc<InnerHostConfig>,
        inner_global_status: Arc<InnerGlobalStatus>,
        inner_api: Arc<InnerAPI>,
        task_desc: types::TaskDescription,
        revid: u64,
        mut listener: mpsc::UnboundedReceiver<TaskCommand>)
    {
        let mut shutdown_announce_handle = None;

        let mut desc = task_desc;
        let mut revid = revid;

        let mut last_run_result = TaskQueryStatus::NoRun;
        let mut last_run_time = chrono::DateTime::<chrono::Utc>::MIN_UTC;

        let task_exec = future::Fuse::terminated();
        let mut task_exec = Box::pin(task_exec);

        // do sleep first.
        // if the cron string is invalid, sleep indefinitely.
        let sleep = if desc.active {
            Self::future_by_cron(&desc.cron)
        } else {
            future::Fuse::terminated()
        };
        let mut sleep = Box::pin(sleep);

        // main loop
        loop {
            futures::select! {
                // sleep has elapsed, sleep returning `Ready(_)` already implies that the cron is valid.
                _ = &mut sleep => {
                    // extract `prefer_bot`
                    let prefer_bot = inner_host_config.prefer_bot;
                    // do the task iff active and not already running.
                    let host_active = {
                        let lock = inner_global_status.host_active.read().await;
                        *lock
                    };
                    if host_active && shutdown_announce_handle.is_none() && desc.active && task_exec.is_terminated() {
                        // get and clone global environment, prepare other values
                        let id = revid;
                        let desc = desc.clone();
                        let host_header = {
                            let lock = inner_global_status.host_header.read().await;
                            (*lock).clone()
                        };
                        let host_deny_ns = {
                            let lock = inner_global_status.host_deny_ns.read().await;
                            (*lock).clone()
                        };
                        let host_timeout = {
                            let lock = inner_global_status.host_timeout.read().await;
                            *lock
                        };
                        let host_query_limit = {
                            let lock = inner_global_status.host_query_limit.read().await;
                            *lock
                        };
                        let api = {
                            let lock = inner_api.api.read().await;
                            (*lock).clone()
                        };
                        let siteinfo = {
                            let lock = inner_api.siteinfo.read().await;
                            (*lock).clone()
                        };
                        let title_codec =  mwtitle::TitleCodec::from_site_info(siteinfo).unwrap();
                        let has_bot_flag = {
                            let lock = inner_api.has_bot_flag.read().await;
                            *lock
                        };

                        task_exec.set(async move {
                            // cloned host configs, copied `id`, cloned `desc`, cloned `api` and new `title_codec` are moved.
                            let types::TaskDescription { expr, eager, config, output, .. } = desc;
                            let timeout = if let Some(config) = config {
                                config.timeout.unwrap_or(host_timeout)
                            } else { host_timeout };
                            let query_limit = if let Some(config) = config {
                                config.query_limit.unwrap_or(host_query_limit)
                            } else { host_query_limit };

                            let provider_api = api.clone();
                            let provider_title_codec = title_codec.clone();
                            let provider = provider::api::APIDataProvider::new(&provider_api, &provider_title_codec);
                            task::TaskExec {
                                id: Some(id),
                                query: expr,
                                timeout,
                                query_limit,
                                output,
                                eager,
                                deny_ns: host_deny_ns,
                                header: host_header,
                                api,
                                title_codec,
                                with_bot_flag: prefer_bot && has_bot_flag,
                                provider: &provider,
                            }.execute(false).await
                        }.fuse());
                        last_run_result = TaskQueryStatus::Running;
                        last_run_time = chrono::Utc::now();
                    }
                    // reset sleep
                    if desc.active && shutdown_announce_handle.is_none() {
                        sleep.set(Self::future_by_cron(&desc.cron));
                    } else {
                        sleep.set(future::Fuse::terminated());
                    }
                },
                summary = &mut task_exec => {
                    // received a summary from the task execution future.
                    last_run_result = TaskQueryStatus::Finished(summary);
                    task_exec.set(future::Fuse::terminated());
                },
                cmd = listener.select_next_some() => {
                    // channel is alive and there are commands
                    match cmd {
                        TaskCommand::Shutdown { receipt } => {
                            // shutting down
                            if shutdown_announce_handle.is_some() {
                                if receipt.send(Err(HostError::IsShuttingDown)).is_err() {
                                    event!(Level::WARN, revid, "task command `shutdown` receipt not sent, receiver hung up.");
                                }
                            } else {
                                // set handle
                                shutdown_announce_handle = Some(receipt);
                                // kill running task
                                task_exec.set(future::Fuse::terminated());
                                listener.close();
                                // terminate sleeping future
                                sleep.set(future::Fuse::terminated());
                            }
                        },
                        TaskCommand::UpdateTaskConfig { new_config, new_revid, receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if new_revid > revid {
                                // update configuration
                                desc = new_config;
                                revid = new_revid;
                                // check new cron
                                if desc.active {
                                    sleep.set(Self::future_by_cron(&desc.cron));
                                } else {
                                    sleep.set(future::Fuse::terminated());
                                }
                                Ok(())
                            } else {
                                // refuse to change anything.
                                Err(HostError::TaskDescriptionNotNewer(revid))
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, revid, "task command `update` receipt not sent, receiver hung up.");
                            }
                        },
                        TaskCommand::AbortRunning { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if !task_exec.is_terminated() {
                                task_exec.set(future::Fuse::terminated());
                                last_run_result = TaskQueryStatus::Aborted;
                                Ok(())
                            } else {
                                Err(HostError::NotRunning)
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, revid, "task command `abort` receipt not sent, receiver hung up.");
                            }
                        },
                        TaskCommand::RunNow { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else if !task_exec.is_terminated() {
                                Err(HostError::AlreadyRunning)
                            } else {
                                // reset sleep timer such that it expires immediately.
                                sleep.set(time::sleep(time::Duration::from_nanos(0)).fuse());
                                Ok(())
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, revid, "task command `runnow` receipt not sent, receiver hung up.");
                            }
                        },
                        TaskCommand::QueryStatus { receipt } => {
                            let outcome = if shutdown_announce_handle.is_some() {
                                Err(HostError::IsShuttingDown)
                            } else {
                                // return details on the task.
                                let frame = TaskStatus {
                                    revid,
                                    task_desc: desc.clone(),
                                    last_run_status: last_run_result.clone(),
                                    last_run_time,
                                };
                                Ok(frame)
                            };
                            if receipt.send(outcome).is_err() {
                                event!(Level::WARN, revid, "task command `query` receipt not sent, receiver hung up.");
                            }
                        },
                    }
                }
                complete => break,
            }
        }

        // there should be a shutdown announce handle
        if let Some(announce_handle) = shutdown_announce_handle {
            // send result
            if announce_handle.send(Ok(())).is_err() {
                event!(Level::WARN, revid, "task command `shutdown` receipt not sent, receiver hung up.");
            }
        } else {
            event!(Level::WARN, revid, "task routine unexpectedly exited.");
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskQueryStatus {
    /// No task was run yet
    NoRun,
    /// The task was running
    Running,
    /// The task was forcefully aborted. This is different from timeout.
    Aborted,
    /// The task was finished, with corresponding results.
    Finished(task::Summary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStatus {
    pub revid: u64,
    pub task_desc: types::TaskDescription,
    pub last_run_status: TaskQueryStatus,
    pub last_run_time: chrono::DateTime<chrono::Utc>,
}
