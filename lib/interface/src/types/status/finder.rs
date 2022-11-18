//! Interface types for task finder.

use std::collections;
use chrono::{DateTime, Utc};
#[cfg(feature="use_serde")]
use serde::{Serialize, Deserialize};
#[cfg(feature="use_serde")]
use serde_with::serde_as;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="use_serde",
    serde_as,
    derive(Serialize, Deserialize),
)]
pub struct PageListBotTaskFinderStatus {
    pub last_run_status: PageListBotTaskFinderRoutineStatus,
    #[cfg_attr(feature="use_serde", serde_as(as = "DisplayFromStr"))]
    pub last_run_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotTaskFinderRoutineStatus {
    /// No finder was run yet
    NoRun,
    /// The finder process was running
    Running,
    /// The finder process was forcefully aborted.
    Aborted,
    /// The finder process was finished, with corresponding results.
    Finished(PageListBotTaskFinderSummary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotTaskFinderSummary {
    /// The finder failed with fetching global configuration.
    GlobalConfigFailed(String),
    /// The finder failed with fetching task list.
    TaskListFailed(String),
    /// Success
    Success(collections::HashMap<u32, PageListBotTaskChange>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotTaskChange {
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
}
