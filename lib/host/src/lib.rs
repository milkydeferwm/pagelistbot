//! Page List Bot task host.
//! 
//! A task host holds an API instance, and is responsible for API renewal, task discovery and task dispatching.
//! 
//! One task host is tied to one MediaWiki-powered site. In a wiki farm like Wikimedia, there are multiple task hosts, one per site.
//! 
//! Task hosts are hosted by bot host.

// TODO: once this is stable, remove this.
#![feature(hash_drain_filter)]

use core::fmt;
use std::{collections, error, sync::Arc};
use futures::channel::mpsc;
use tokio::{sync, task};
use tracing::{event, Level};
use interface::types::ast::NumberOrInf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct InnerHostConfig {
    /// Username used by the host.
    username: String,
    /// Password used by the host.
    password: String,
    /// The target MediaWiki site API endpoint.
    api_endpoint: String,
    /// Where does the on-site configuration lies.
    onsite_config: String,
    /// Should the bot use bot flag when editing, if possible.
    prefer_bot: bool,
}

#[derive(Debug)]
struct InnerGlobalStatus {
    /// Is the bot globally activated?
    host_active: sync::RwLock<bool>,
    /// The header template used by output pages.
    host_header: sync::RwLock<String>,
    /// List of namespaces the bot will unconditionally refuse to write to.
    host_deny_ns: sync::RwLock<collections::BTreeSet<i32>>,
    /// Global default query execution timeout.
    host_timeout: sync::RwLock<u64>,
    /// Global default query item limit.
    host_query_limit: sync::RwLock<NumberOrInf<usize>>,
}

#[derive(Debug)]
/// Cross-thread shared API object.
struct InnerAPI {
    /// Shared API client.
    api: sync::RwLock<mwapi::Client>,
    /// Shared siteinfo.
    siteinfo: sync::RwLock<mwtitle::SiteInfo>,
    /// Does the bot have a `bot` flag?
    has_bot_flag: sync::RwLock<bool>,
    /// Does the bot have a `apihighlimits` flag?
    has_apihighlimits_flag: sync::RwLock<bool>,
}

#[derive(Debug)]
struct TaskMapTaskInfo {
    pub tx: mpsc::UnboundedSender<routine::TaskCommand>,
    pub handle: task::JoinHandle<()>,
    pub pageid: u32,
    pub latest_rev: u64,
}

/// safe guard against unexcepted dropping
impl Drop for TaskMapTaskInfo {
    fn drop(&mut self) {
        self.tx.close_channel();
        if !self.handle.is_finished() {
            self.handle.abort();
            event!(Level::WARN, self.pageid, "force killed.");
        }
    }
}

/// The public facing task host.
#[derive(Debug)]
pub struct Host {
    // Rust gurantees that struct field declared first gets destructed first.
    // thread join handles and corresponding channels
    finder_tx: mpsc::UnboundedSender<routine::FinderCommand>,
    finder_handle: task::JoinHandle<()>,
    refresher_tx: mpsc::UnboundedSender<routine::RefresherCommand>,
    refresher_handle: task::JoinHandle<()>,
    // inner objects shared across tokio-spawned threads.
    inner_host_config: Arc<InnerHostConfig>,
    inner_global_status: Arc<InnerGlobalStatus>,
    inner_task_map: Arc<sync::RwLock<collections::HashMap<u32, TaskMapTaskInfo>>>,
    inner_api: Arc<InnerAPI>,
}

impl Drop for Host {
    fn drop(&mut self) {
        // close channels
        self.finder_tx.close_channel();
        self.refresher_tx.close_channel();
        // kill processes
        if !self.finder_handle.is_finished() {
            self.finder_handle.abort();
            event!(Level::WARN, "finder routine force killed.");
        }
        if !self.refresher_handle.is_finished() {
            self.refresher_handle.abort();
            event!(Level::WARN, "refresher routine force killed.");
        }
    }
}

/// List of errors a host may emit.
#[derive(Debug, Clone)]
pub enum HostError {
    /// There is a problem creating the new API client.
    APIError(mwapi::Error),
    /// The requested command cannot be fulfilled because the the target is shutting down.
    IsShuttingDown,
    /// The requested "abort" command cannot be fulfilled because the target is not running.
    NotRunning,
    /// The requested "run now" command cannot be fulfilled because the target is already running.
    AlreadyRunning,
    /// `task_rt`-specific. The requested "update task description" command cannot be fulfilled because the current description revision is newer.
    TaskDescriptionNotNewer(u64),
    /// The requested task does not exist.
    TaskDoesNotExist,
    /// The target process hung up. This should not happen.
    HungUp,
}

impl error::Error for HostError {}
impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::APIError(e) => write!(f, "API error: {}", e),
            Self::IsShuttingDown => write!(f, "the requested entity is already shutting down"),
            Self::NotRunning => write!(f, "the requested routine is already at sleep"),
            Self::AlreadyRunning => write!(f, "the requested routine is already running"),
            Self::TaskDescriptionNotNewer(newer) => write!(f, "the requested task routine has a newer task description version: {}", newer),
            Self::TaskDoesNotExist => write!(f, "the requested task does not exist"),
            Self::HungUp => write!(f, "the requested entity unexpected hung up"),
        }
    }
}

impl From<HostError> for interface::PageListBotError {
    fn from(value: HostError) -> Self {
        Self::HostError(value.to_string())
    }
}

// Other module uses
pub mod host_impl;
mod host_util;
mod functional;
mod routine;
