//! Page List Bot task finder methods. Finder executions are dispatched by `Host`'s `rt_finder`.

use std::collections;
use std::sync::Arc;
use crate::{Host, InnerHostConfig, InnerGlobalStatus, InnerAPI, TaskMapTaskInfo, HostError};
use crate::routine::TaskCommand;

use interface::types::site::{HostConfig, TaskDescription};
use interface::types::status::finder::{PageListBotTaskChange, PageListBotTaskFinderSummary};
use futures::{prelude::*, channel::{mpsc, oneshot}, stream};
use mwapi_responses::query;
use tokio::time;
use tracing::{event, Level};

// Ideally this should be derived from `mwapi_responses`, but it does not provide everything we want.
/// A response object from API query.
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
struct PageContentResponse {
    #[serde(default)]
    pub batchcomplete: bool,
    #[serde(rename = "continue")]
    #[serde(default)]
    pub continue_: collections::HashMap<String, String>,
    pub query: PageContentResponseBody,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PageContentResponseBody {
    pub pages: Vec<PageContentResponseBodyPagesItem>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PageContentResponseBodyPagesItem {
    pub revisions: Vec<PageContentResponseBodyPagesItemRevisionsItem>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PageContentResponseBodyPagesItemRevisionsItem {
    pub revid: u64,
    pub slots: PageContentResponseBodyPagesItemRevisionsItemSlots,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PageContentResponseBodyPagesItemRevisionsItemSlots {
    pub main: PageContentResponseBodyPagesItemRevisionsItemSlotsMain,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PageContentResponseBodyPagesItemRevisionsItemSlotsMain {
    pub contentmodel: String,
    pub contentformat: String,
    pub content: String,
}

#[query(
    prop="info",
)]
struct PrefixIndexResponse;


pub(crate) struct FinderExec<'exec> {
    pub host_config: Arc<InnerHostConfig>,
    pub global_status: Arc<InnerGlobalStatus>,
    pub inner_api: Arc<InnerAPI>,
    pub task_map: &'exec mut collections::HashMap<u32, TaskMapTaskInfo>,
}

impl<'exec> FinderExec<'exec> {
    /// Fetch a page's content. The page content should be in JSON.
    async fn get_content_by_title<T>(api: &mwapi::Client, title: &str) -> Result<(u64, T), FinderError>
    where
        T: serde::de::DeserializeOwned,
    {
        let params = collections::HashMap::from_iter([
            ("action".to_string(), "query".to_string()),
            ("prop".to_string(), "revisions".to_string()),
            ("titles".to_string(), title.to_owned()),
            ("rvslots".to_string(), "main".to_string()),
            ("rvprop".to_string(), "content|contentmodel|ids".to_string()),
        ]);
        let resp: PageContentResponse = api.get(params).await
            .map_err(FinderError::APIError)?;
        let item = resp.query.pages.get(0).ok_or(FinderError::MalformedResponse)?;
        let rev = item.revisions.get(0).ok_or(FinderError::MalformedResponse)?;
        // it must be a JSON page... A wikitext page containing valid JSON is still invalid.
        if (rev.slots.main.contentmodel == "json") && (rev.slots.main.contentformat == "application/json") {
            let content = serde_json::from_str(&rev.slots.main.content).map_err(|e| FinderError::CondentJsonDeserializeError(Arc::new(e)))?;
            let revid = rev.revid;
            Ok((revid, content))
        } else {
            Err(FinderError::ContentModelNotJson)
        }
    }

    /// Fetch a page's content. The page content should be in JSON.
    async fn get_content_by_pageid<T>(api: &mwapi::Client, pageid: u32) -> Result<(u64, T), FinderError>
    where
        T: serde::de::DeserializeOwned,
    {
        let params = collections::HashMap::from_iter([
            ("action".to_string(), "query".to_string()),
            ("prop".to_string(), "revisions".to_string()),
            ("pageids".to_string(), pageid.to_string()),
            ("rvslots".to_string(), "main".to_string()),
            ("rvprop".to_string(), "content|contentmodel|ids".to_string()),
        ]);
        let resp: PageContentResponse = api.get(params).await
            .map_err(FinderError::APIError)?;
        let item = resp.query.pages.get(0).ok_or(FinderError::MalformedResponse)?;
        let rev = item.revisions.get(0).ok_or(FinderError::MalformedResponse)?;
        // it must be a JSON page... A wikitext page containing valid JSON is still invalid.
        if (rev.slots.main.contentmodel == "json") && (rev.slots.main.contentformat == "application/json") {
            let content = serde_json::from_str(&rev.slots.main.content).map_err(|e| FinderError::CondentJsonDeserializeError(Arc::new(e)))?;
            let revid = rev.revid;
            Ok((revid, content))
        } else {
            Err(FinderError::ContentModelNotJson)
        }
    }

    /// Fetch a page's all subpages, whose content model is JSON. Returns the pageid and lastrevisionid.
    async fn get_tasks(api: &mwapi::Client, title_codec: &mwtitle::TitleCodec, root: &str) -> Result<collections::BTreeMap<u32, u64>, FinderError> {
        let title = title_codec.new_title(root).map_err(|e| FinderError::APIError(e.into()))?;
        let params = collections::HashMap::from_iter([
            ("action".to_string(), "query".to_string()),
            ("prop".to_string(), "info".to_string()),
            ("generator".to_string(), "allpages".to_string()),
            ("gapprefix".to_string(), title.dbkey().to_owned()),
            ("gapnamespace".to_string(), title.namespace().to_string()),
            ("gaplimit".to_string(), "max".to_string()),
            ("gapfilterredir".to_string(), "nonredirects".to_string()),
        ]);
        let mut continue_ = collections::HashMap::new();
        let mut result = collections::BTreeMap::new();
        loop {
            let mut params = params.clone();
            params.extend(continue_.into_iter());
            let resp: PrefixIndexResponse = api.get(params).await.map_err(FinderError::APIError)?;
            continue_ = resp.continue_.clone();
            // filter and insert to result
            for item in resp.query.pages.iter().filter(|&i| i.contentmodel == "json") {
                // pages from a `allpages` API query should all exist.
                let pageid = item.pageid.ok_or(FinderError::MalformedResponse)?;
                let lastrevid = item.lastrevid.ok_or(FinderError::MalformedResponse)?;
                result.insert(pageid, lastrevid);
            }
            if continue_.is_empty() {
                break;
            }
        }
        Ok(result)
    }

    /// drop a running task
    async fn drop_task(task: TaskMapTaskInfo) -> FinderTaskChange {
        event!(Level::INFO, task.pageid, "will drop running task.");
        // send shutdown instruction, with 2-sec timeout
        let (r_tx, r_rx) = oneshot::channel();
        if task.tx.unbounded_send(TaskCommand::Shutdown { receipt: r_tx }).is_err() {
            // cannot send, log
            event!(Level::WARN, task.pageid, "cannot send shutdown signal to running task, receiver hung up.");
        } else {
            let result = time::timeout(time::Duration::from_secs(2), r_rx).await;
            if result.is_err() {
                // shutdown timeout
                event!(Level::WARN, task.pageid, "running task failed to react in 2 secs.");
            } else {
                let result = result.unwrap();
                if result.is_err() {
                    // should not happen
                    event!(Level::WARN, task.pageid, "running task dropped the sender prematurely.");
                } else {
                    let result = result.unwrap();
                    if let Err(e) = result {
                        // rt_task only returns a `TaskIsShuttingDown` error when processing a shutdown command.
                        // there must be another entity requesting shut down, we do not force kill here.
                        event!(Level::WARN, task.pageid, ?e, "running task returned an error.");
                    }
                }
            }
        }
        // if code reaches here, normal shutdown procedure failed.
        // process will be force aborted when `task` goes out of scope.
        FinderTaskChange::Killed
    }

    async fn update_task(api: &mwapi::Client, task: &mut TaskMapTaskInfo) -> FinderTaskChange {
        event!(Level::INFO, task.pageid, "will update running task.");
        // fetch task content
        let task_desc = Self::get_content_by_pageid::<TaskDescription>(api, task.pageid).await;
        if let Err(e) = task_desc {
            // whatever reason the task config failed to retrive, kill it.
            event!(Level::WARN, task.pageid, ?e, "cannot fetch task description.");
            FinderTaskChange::ToKill
        } else {
            let (new_rev, task_desc) = task_desc.unwrap();
            // send a update task signal
            let (u_tx, u_rx) = oneshot::channel();
            if task.tx.unbounded_send(TaskCommand::UpdateTaskConfig { new_config: task_desc.clone(), new_revid: new_rev, receipt: u_tx }).is_err() {
                // task failed to send
                event!(Level::WARN, task.pageid, "cannot send update task signal to running task, receiver hung up. will restart running task.");
                FinderTaskChange::ToRestart(new_rev, task_desc)
            } else {
                let result = time::timeout(time::Duration::from_secs(2), u_rx).await;
                if result.is_err() {
                    // update timeout
                    event!(Level::WARN, task.pageid, "running task failed to react in 2 secs. will restart running task.");
                    FinderTaskChange::ToRestart(new_rev, task_desc)
                } else {
                    let result = result.unwrap();
                    if result.is_err() {
                        // should not happen
                        event!(Level::WARN, task.pageid, "running task dropped the sender prematurely. will restart running task.");
                        FinderTaskChange::ToRestart(new_rev, task_desc)
                    } else {
                        let result = result.unwrap();
                        let newer_revid = if let Err(e) = result {
                            event!(Level::WARN, task.pageid, ?e, "running task refused to update.");
                            if let HostError::TaskDescriptionNotNewer(newer_revid) = e { newer_revid } else { new_rev }
                        } else { new_rev };
                        // nevertheless (is shutdown or already newer), mark as updated
                        task.latest_rev = newer_revid;
                        FinderTaskChange::Updated
                    }
                }
            }
        }
    }

    fn dispatch_task(&self, pageid: u32, revid: u64, task_desc: TaskDescription) -> TaskMapTaskInfo {
        let (tx, rx) = mpsc::unbounded();
        let handle = tokio::spawn(Host::task_rt(
            self.host_config.clone(),
            self.global_status.clone(),
            self.inner_api.clone(),
            task_desc,
            revid,
            rx));
        TaskMapTaskInfo {
            tx,
            handle,
            pageid,
            latest_rev: revid,
        }
    }

    /// Execute the finder execution.
    pub async fn execute(&mut self) -> FinderSummary {
        // extract API and TitleCodec
        let api = {
            let lock = self.inner_api.api.read().await;
            (*lock).clone()
        };
        let siteinfo = {
            let lock = self.inner_api.siteinfo.read().await;
            (*lock).clone()
        };
        let title_codec = mwtitle::TitleCodec::from_site_info(siteinfo).unwrap();

        // fetch latest config
        let global_config = Self::get_content_by_title::<HostConfig>(&api, &self.host_config.onsite_config).await;
        if let Err(e) = global_config { // be conservative, set global active to false
            let mut g_active = self.global_status.host_active.write().await;
            *g_active = false;
            return FinderSummary::GlobalConfigFailed(e);
        }
        // update global configs
        let (_, global_config) = global_config.unwrap();
        let HostConfig { active, task_dir, header, deny_ns, default_task_config, .. } = global_config;
        {
            let mut g_active = self.global_status.host_active.write().await;
            *g_active = active;
        }
        {
            let mut g_header = self.global_status.host_header.write().await;
            *g_header = header;
        }
        {
            let mut g_deny_ns = self.global_status.host_deny_ns.write().await;
            *g_deny_ns = deny_ns;
        }
        {
            let mut g_timeout = self.global_status.host_timeout.write().await;
            *g_timeout = default_task_config.timeout;
        }
        {
            let mut g_query_limit = self.global_status.host_query_limit.write().await;
            *g_query_limit = default_task_config.query_limit;
        }

        // get list of tasks
        let tasks = Self::get_tasks(&api, &title_codec, &task_dir).await;
        if let Err(e) = tasks { // be conservative, set global active to false
            let mut g_active = self.global_status.host_active.write().await;
            *g_active = false;
            return FinderSummary::TaskListFailed(e);
        }
        let tasks = tasks.unwrap();

        // prepare output
        // initialize all current task map entries as unchanged.
        let mut update_result: collections::HashMap<u32, FinderTaskChange> = collections::HashMap::from_iter(self.task_map.iter().map(|(&pid, _)| (pid, FinderTaskChange::NoChange)));
        // drop tasks that no longer exists.
        {
            let dropped_futures = (*self.task_map).drain_filter(|k, _| !tasks.contains_key(k))
                .map(|(pid, v)| async move { (pid, Self::drop_task(v).await) })
                .collect::<stream::FuturesUnordered<_>>();
            update_result.extend(dropped_futures.collect::<Vec<_>>().await);
        }
        // update tasks that has newer revid.
        {
            let updated_futures = (*self.task_map).iter_mut()
                .filter(|(&k, v)| *(tasks.get(&k).unwrap()) > v.latest_rev)
                .map(|(&pid, v)| {
                    let api = api.to_owned();
                    async move { (pid, Self::update_task(&api, v).await) }
                })
                .collect::<stream::FuturesUnordered<_>>();
            update_result.extend(updated_futures.collect::<Vec<_>>().await);
        }
        // dispatch tasks that are new.
        {
            let new_task_descs = tasks.iter()
                .filter(|(&pid, _)| !self.task_map.contains_key(&pid))
                .map(|(&pid, _)| {
                    let api = api.to_owned();
                    async move { (pid, Self::get_content_by_pageid::<TaskDescription>(&api, pid).await) }
                })
                .collect::<stream::FuturesUnordered<_>>();
            for (pid, task_desc) in new_task_descs.collect::<Vec<_>>().await {
                if let Ok((revid, task_desc)) = task_desc {
                    // dispatch new task
                    let task_info = self.dispatch_task(pid, revid, task_desc);
                    self.task_map.insert(pid, task_info);
                    update_result.insert(pid, FinderTaskChange::Created);
                } else {
                    let e = task_desc.unwrap_err();
                    event!(Level::WARN, pid, ?e, "task creation skipped because of error.");
                    update_result.insert(pid, FinderTaskChange::Skipped);
                }
            }
        }
        // kill tasks that are marked `ToKill`.
        {
            let dropped_futures = (*self.task_map).drain_filter(|k, _| Some(&FinderTaskChange::ToKill) == update_result.get(k))
                .map(|(pid, v)| async move { (pid, Self::drop_task(v).await) })
                .collect::<stream::FuturesUnordered<_>>();
            update_result.extend(dropped_futures.collect::<Vec<_>>().await);
        }
        // kill and dispatch tasks that are marked `ToRestart`
        {
            // kill
            let dropped_futures = (*self.task_map).drain_filter(|k, _| matches!(update_result.get(k), Some(&FinderTaskChange::ToRestart(_, _))))
                .map(|(pid, v)| async move { (pid, Self::drop_task(v).await) })
                .collect::<stream::FuturesUnordered<_>>();
            dropped_futures.collect::<Vec<_>>().await;
            // respawn
            let to_respawn = update_result.drain_filter(|_, v| matches!(*v, FinderTaskChange::ToRestart(_, _))).collect::<collections::HashMap<_, _>>();
            for (pid, v) in to_respawn {
                if let FinderTaskChange::ToRestart(revid, task_desc) = v {
                    let task_info = self.dispatch_task(pid, revid, task_desc);
                    (*self.task_map).insert(pid, task_info);
                    update_result.insert(pid, FinderTaskChange::Restarted);
                } else { unreachable!() }
            }
        }
        let update_result: collections::HashMap<u32, PageListBotTaskChange> = collections::HashMap::from_iter(update_result.into_iter().map(|(k, v)| (k, v.into())));
        FinderSummary::Success(update_result)
    }
}

/// The summary of a finder routine.
/// This logs how 
#[derive(Debug, Clone)]
pub enum FinderSummary {
    /// The finder failed with fetching global configuration.
    GlobalConfigFailed(FinderError),
    /// The finder failed with fetching task list.
    TaskListFailed(FinderError),
    /// Success
    Success(collections::HashMap<u32, PageListBotTaskChange>),
}

impl From<FinderSummary> for PageListBotTaskFinderSummary {
    fn from(value: FinderSummary) -> Self {
        match value {
            FinderSummary::GlobalConfigFailed(e) => Self::GlobalConfigFailed(e.to_string()),
            FinderSummary::TaskListFailed(e) => Self::TaskListFailed(e.to_string()),
            FinderSummary::Success(r) => Self::Success(r),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinderTaskChange {
    /// This task is not changed.
    NoChange,
    /// This task is killed.
    Killed,
    /// This task is updated.
    Updated,
    /// This task is created.
    Created,
    /// Something is wrong with the task and the task is restarted.
    Restarted,
    /// INTERNAL: The task needs to be killed.
    ToKill,
    /// INTERNAL: The task needs to be restarted.
    ToRestart(u64, TaskDescription),
    /// The task creation is skipped.
    Skipped,
}

impl From<FinderTaskChange> for PageListBotTaskChange {
    fn from(value: FinderTaskChange) -> Self {
        match value {
            FinderTaskChange::NoChange => Self::NoChange,
            FinderTaskChange::Killed => Self::Killed,
            FinderTaskChange::Updated => Self::Updated,
            FinderTaskChange::Created => Self::Created,
            FinderTaskChange::Restarted => Self::Restarted,
            FinderTaskChange::Skipped => Self::Skipped,
            _ => unreachable!(),
        }
    }
}

/// Errors made by Finder.
#[derive(Debug, Clone)]
pub enum FinderError {
    /// There is an error with MediaWiki API query and deserializing the response.
    APIError(mwapi::Error),
    /// The response deserialized, but cannot extract the thing we want.
    MalformedResponse,
    /// The query is successful, but the target page is not a JSON page.
    ContentModelNotJson,
    /// The query is successful and the page is a JSON page, but cannot deserialize the content into requred struct.
    CondentJsonDeserializeError(Arc<serde_json::Error>),
}

impl std::error::Error for FinderError {}
impl core::fmt::Display for FinderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::APIError(e) => write!(f, "{}", e),
            Self::MalformedResponse => write!(f, "the API response is malformed"),
            Self::ContentModelNotJson => write!(f, "the config page does not have a JSON content model"),
            Self::CondentJsonDeserializeError(e) => write!(f, "{}", e),
        }
    }
}
