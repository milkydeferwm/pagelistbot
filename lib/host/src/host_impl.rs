//! Host related functions.

use std::{collections, sync::Arc};
use crate::{Host, InnerHostConfig, InnerGlobalStatus, InnerAPI, HostError};
use crate::functional::refresh::{UserInfoResponse, UserInfo};
use crate::routine;

use interface::types::ast::NumberOrInf;
use futures::{prelude::*, channel::oneshot, stream};
use tokio::{sync, time};
use tracing::{event, Level};

impl Host {
    /// Create a new `Host` if successful.
    pub async fn try_new(username: &str, password: &str, api_endpoint: &str, onsite_config: &str, prefer_bot: bool) -> Result<Host, HostError> {
        // create the API client, use it to initialize the `siteinfo` and `has_bot_flag`.
        let api = mwapi::Client::builder(api_endpoint)
            .set_botpassword(username, password)
            .set_errorformat(mwapi::ErrorFormat::default())
            .set_assert(mwapi::Assert::User) // make sure we are signed in. no need to check bot flag.
            .set_user_agent(&format!("Page List Bot v{} / User:{} / {}", env!("CARGO_PKG_VERSION"), &username, env!("CARGO_PKG_REPOSITORY")))
            .build()
            .await
            .map_err(HostError::APIError)?;
        // use the api to query for siteinfo
        let siteinfo: mwtitle::SiteInfo = {
            let params = collections::HashMap::from_iter([
                ("action", "query"),
                ("meta", "siteinfo"),
                ("siprop", "general|namespaces|namespacealiases|interwikimap")
            ]);
            api.get::<_, mwtitle::SiteInfoResponse>(params)
                .await
                .map_err(HostError::APIError)
                .map(|e| e.query)
        }?;
        let userinfo: UserInfo = {
            let params = collections::HashMap::from_iter([
                ("action", "query"),
                ("meta", "userinfo"),
                ("uiprop", "rights"),
            ]);
            api.get::<_, UserInfoResponse>(params)
                .await
                .map_err(HostError::APIError)
                .map(|e| e.query.userinfo)
        }?;

        // assemble inner objects
        let host_config = Arc::new(InnerHostConfig {
            username: username.to_owned(),
            password: password.to_owned(),
            api_endpoint: api_endpoint.to_owned(),
            onsite_config: onsite_config.to_owned(),
            prefer_bot,
        });
        let global_status = Arc::new(InnerGlobalStatus {
            host_active: sync::RwLock::new(false),
            host_header: sync::RwLock::new(String::new()),
            host_deny_ns: sync::RwLock::new(collections::BTreeSet::new()),
            host_timeout: sync::RwLock::new(0),
            host_query_limit: sync::RwLock::new(NumberOrInf::default()),
        });
        let task_map = Arc::new(sync::RwLock::new(collections::HashMap::new()));
        let inner_api = Arc::new(InnerAPI {
            api: sync::RwLock::new(api),
            siteinfo: sync::RwLock::new(siteinfo),
            has_bot_flag: sync::RwLock::new(userinfo.rights.contains("bot")),
        });
        // assemble handlers and senders
        let (finder_handle, finder_tx) = Host::start_task_finder(host_config.clone(), global_status.clone(), task_map.clone(), inner_api.clone());
        let (refresher_handle, refresher_tx) = Host::start_api_refresher(host_config.clone(), inner_api.clone());

        let new_host = Self {
            inner_host_config: host_config,
            inner_global_status: global_status,
            inner_task_map: task_map,
            inner_api,
            finder_handle,
            finder_tx,
            refresher_handle,
            refresher_tx,
        };

        Ok(new_host)
    }

    /// Get a list of active running tasks. Result will be any collection that can hold `u32`, the pageid's type.
    pub async fn get_active_tasks<T>(&self) -> T
    where
        T: FromIterator<u32>
    {
        let task_map = self.inner_task_map.read().await;
        T::from_iter(task_map.keys().copied())
    }

    /// Inspect into a task. Get the task's running status.
    pub async fn inspect_task(&self, pageid: u32) -> Result<routine::TaskStatus, HostError> {
        let task_map = self.inner_task_map.read().await;
        if let Some(taskinfo) = task_map.get(&pageid) {
            let (tx, rx) = oneshot::channel();
            if taskinfo.tx.unbounded_send(routine::TaskCommand::QueryStatus { receipt: tx }).is_err() {
                return Err(HostError::HungUp);
            }
            time::timeout(time::Duration::from_secs(2), rx).await
                .map_err(|_| HostError::HungUp)? // timeout
                .map_err(|_| HostError::HungUp)? // cancelled
        } else {
            Err(HostError::TaskDoesNotExist)
        }
    }

    /// Inspect the task finder's status.
    pub async fn inspect_finder(&self) -> Result<routine::FinderStatus, HostError> {
        let (tx, rx) = oneshot::channel();
        if self.finder_tx.unbounded_send(routine::FinderCommand::QueryStatus { receipt: tx }).is_err() {
            return Err(HostError::HungUp);
        }
        time::timeout(time::Duration::from_secs(2), rx).await
            .map_err(|_| HostError::HungUp)? // timeout
            .map_err(|_| HostError::HungUp)? // cancelled
    }

    /// Inspect the API refresher's status.
    pub async fn inspect_refresher(&self) -> Result<routine::RefresherStatus, HostError> {
        let (tx, rx) = oneshot::channel();
        if self.refresher_tx.unbounded_send(routine::RefresherCommand::QueryStatus { receipt: tx }).is_err() {
            return Err(HostError::HungUp);
        }
        time::timeout(time::Duration::from_secs(2), rx).await
            .map_err(|_| HostError::HungUp)? // timeout
            .map_err(|_| HostError::HungUp)? // cancelled
    }

    /// Wake up a task.
    pub async fn wakeup_task(&self, pageid: u32) -> Result<(), HostError> {
        let task_map = self.inner_task_map.read().await;
        if let Some(taskinfo) = task_map.get(&pageid) {
            let (tx, rx) = oneshot::channel();
            if taskinfo.tx.unbounded_send(routine::TaskCommand::RunNow { receipt: tx }).is_err() {
                return Err(HostError::HungUp);
            }
            time::timeout(time::Duration::from_secs(2), rx).await
                .map_err(|_| HostError::HungUp)? // timeout
                .map_err(|_| HostError::HungUp)? // cancelled
        } else {
            Err(HostError::TaskDoesNotExist)
        }
    }

    /// Wakeup the task finder.
    pub async fn wakeup_finder(&self) -> Result<(), HostError> {
        let (tx, rx) = oneshot::channel();
        if self.finder_tx.unbounded_send(routine::FinderCommand::RunNow { receipt: tx }).is_err() {
            return Err(HostError::HungUp);
        }
        time::timeout(time::Duration::from_secs(2), rx).await
            .map_err(|_| HostError::HungUp)? // timeout
            .map_err(|_| HostError::HungUp)? // cancelled
    }

    /// Wakeup the API refresher.
    pub async fn wakeup_refresher(&self) -> Result<(), HostError> {
        let (tx, rx) = oneshot::channel();
        if self.refresher_tx.unbounded_send(routine::RefresherCommand::RunNow { receipt: tx }).is_err() {
            return Err(HostError::HungUp);
        }
        time::timeout(time::Duration::from_secs(2), rx).await
            .map_err(|_| HostError::HungUp)? // timeout
            .map_err(|_| HostError::HungUp)? // cancelled
    }

    /// Abort a running task, put it back to sleep.
    pub async fn abort_running_task(&self, pageid: u32) -> Result<(), HostError> {
        let task_map = self.inner_task_map.read().await;
        if let Some(taskinfo) = task_map.get(&pageid) {
            let (tx, rx) = oneshot::channel();
            if taskinfo.tx.unbounded_send(routine::TaskCommand::AbortRunning { receipt: tx }).is_err() {
                return Err(HostError::HungUp);
            }
            time::timeout(time::Duration::from_secs(2), rx).await
                .map_err(|_| HostError::HungUp)? // timeout
                .map_err(|_| HostError::HungUp)? // cancelled
        } else {
            Err(HostError::TaskDoesNotExist)
        }
    }

    /// Abort a running task finder, put it back to sleep.
    pub async fn abort_running_finder(&self) -> Result<(), HostError> {
        let (tx, rx) = oneshot::channel();
        if self.finder_tx.unbounded_send(routine::FinderCommand::AbortRunning { receipt: tx }).is_err() {
            return Err(HostError::HungUp);
        }
        time::timeout(time::Duration::from_secs(2), rx).await
            .map_err(|_| HostError::HungUp)? // timeout
            .map_err(|_| HostError::HungUp)? // cancelled
    }

    /// Abort a running API refresher, put it back to sleep.
    pub async fn abort_running_refresher(&self) -> Result<(), HostError> {
        let (tx, rx) = oneshot::channel();
        if self.refresher_tx.unbounded_send(routine::RefresherCommand::AbortRunning { receipt: tx }).is_err() {
            return Err(HostError::HungUp);
        }
        time::timeout(time::Duration::from_secs(2), rx).await
            .map_err(|_| HostError::HungUp)? // timeout
            .map_err(|_| HostError::HungUp)? // cancelled
    }

    /// Close task finder. This should not be called without restarting it again.
    async fn close_finder(&self) {
        // try to politely ask finder to shut down.
        let (tx, rx) = oneshot::channel();
        if self.finder_tx.unbounded_send(routine::FinderCommand::Shutdown { receipt: tx }).is_err() {
            // assume task finder has hung up. desperate status.
            self.finder_tx.close_channel();
            self.finder_handle.abort();
            event!(Level::WARN, "cannot send shutdown signal to task finder, receiver hung up. force killed.");
        } else {
            // wait for at most two seconds, otherwise we consider finder routine is in desperate status.
            let result = time::timeout(time::Duration::from_secs(2), rx).await;
            if result.is_err() {
                // desprate status.
                self.finder_tx.close_channel();
                self.finder_handle.abort();
                event!(Level::WARN, "task finder failed to react in 2 secs. force killed.");
            } else {
                let result = result.unwrap();
                if result.is_err() {
                    // should not happen
                    self.finder_tx.close_channel();
                    self.finder_handle.abort();
                    event!(Level::WARN, "task finder dropped the sender prematurely. force killed.");
                } else {
                    let result = result.unwrap();
                    if let Err(e) = result {
                        // rt_finder only returns a `IsShuttingDown` error when processing a shutdown command.
                        // there must be another entity requesting shut down, we do not force kill here.
                        event!(Level::WARN, ?e, "task finder returned an error.");
                    }
                }
            }
        }
    }

    /// Close API refresher. This should not be called without restarting it again.
    async fn close_refresher(&self) {
        // try to politely ask refresher to shut down.
        let (tx, rx) = oneshot::channel();
        if self.refresher_tx.unbounded_send(routine::RefresherCommand::Shutdown { receipt: tx }).is_err() {
            // assume API refresher has hung up. desperate status.
            self.refresher_tx.close_channel();
            self.refresher_handle.abort();
            event!(Level::WARN, "cannot send shutdown signal to API refresher, receiver hung up. force killed.");
        } else {
            // wait for at most two seconds, otherwise we consider refresher routine is in desperate status.
            let result = time::timeout(time::Duration::from_secs(2), rx).await;
            if result.is_err() {
                // desprate status.
                self.refresher_tx.close_channel();
                self.refresher_handle.abort();
                event!(Level::WARN, "API refresher failed to react in 2 secs. force killed.");
            } else {
                let result = result.unwrap();
                if result.is_err() {
                    // should not happen
                    self.refresher_tx.close_channel();
                    self.refresher_handle.abort();
                    event!(Level::WARN, "API refresher dropped the sender prematurely. force killed.");
                } else {
                    let result = result.unwrap();
                    if let Err(e) = result {
                        // rt_refresher only returns a `IsShuttingDown` error when processing a shutdown command.
                        // there must be another entity requesting shut down, we do not force kill here.
                        event!(Level::WARN, ?e, "API refresher returned an error.");
                    }
                }
            }
        }
    }

    /// Restart task finder.
    pub async fn restart_finder(&mut self) {
        self.close_finder().await;
        // spawn and dispatch new finder routine.
        let (finder_handle, finder_tx) = Host::start_task_finder(
            self.inner_host_config.clone(),
            self.inner_global_status.clone(),
            self.inner_task_map.clone(),
            self.inner_api.clone());
        self.finder_tx = finder_tx;
        self.finder_handle = finder_handle;
    }

    /// Restart API refresher.
    pub async fn restart_refresher(&mut self) {
        self.close_refresher().await;
        // spawn and dispatch new refresher routine.
        let (refresher_handle, refresher_tx) = Host::start_api_refresher(
            self.inner_host_config.clone(),
            self.inner_api.clone());
        self.refresher_tx = refresher_tx;
        self.refresher_handle = refresher_handle;
    }

    /// Shutdown the `Host`.
    /// This closes task finder and API refresher first. After that, closes every running task.
    /// This would take at most 4 seconds, after that, everything is force dropped.
    pub async fn shutdown(&self) {
        futures::join!(self.close_refresher(), self.close_finder()); // takes at most 2-secs
        let mut task_map = self.inner_task_map.write().await;
        let dropped_tasks = task_map.drain().into_iter().map(|(pid, v)| async move {
            // send shutdown instruction, with 2-sec timeout
            let (r_tx, r_rx) = oneshot::channel();
            if v.tx.unbounded_send(routine::TaskCommand::Shutdown { receipt: r_tx }).is_err() {
                // cannot send, log
                event!(Level::WARN, pid, "cannot send shutdown signal to running task, receiver hung up.");
            } else {
                let result = time::timeout(time::Duration::from_secs(2), r_rx).await;
                if let Ok(result) = result {
                    if let Ok(result) = result {
                        if let Err(e) = result {
                            // rt_task only returns a `IsShuttingDown` error when processing a shutdown command.
                            // there must be another entity requesting shut down, we do not force kill here.
                            event!(Level::WARN, pid, ?e, "running task returned an error.");
                        }
                    } else {
                        // should not happen
                        event!(Level::WARN, pid, "running task dropped the sender prematurely.");
                    }
                } else {
                    // shutdown timeout
                    event!(Level::WARN, pid, "running task failed to react in 2 secs.");
                }
            }
        }).collect::<stream::FuturesUnordered<_>>();
        dropped_tasks.collect::<Vec<_>>().await; // takes at most 2-secs
    }
}
