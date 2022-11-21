//! Interface types for task routine.

use std::collections;
use crate::types::site::TaskDescription;

use chrono::{DateTime, Utc};
#[cfg(feature = "use_serde")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "use_serde")]
use serde_with::serde_as;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde",
    serde_as,
    derive(Serialize, Deserialize),
)]
pub struct PageListBotTaskStatus {
    /// The task's recorded revision id.
    pub revid: u64,
    /// The task's recorded task description.
    pub task_desc: TaskDescription,
    pub last_run_status: PageListBotTaskQueryStatus,
    #[cfg_attr(feature = "use_serde", serde_as(as = "DisplayFromStr"))]
    pub last_run_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotTaskQueryStatus {
    /// No task was run yet
    NoRun,
    /// The task was running
    Running,
    /// The task was forcefully aborted. This is different from timeout.
    Aborted,
    /// The task was finished, with corresponding results.
    Finished(PageListBotTaskQuerySummary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct PageListBotTaskQuerySummary {
    /// The status of the query: if successful, returns the pages found and the warnings, otherwise returns the `PageListBotTaskQueryError` object.
    pub query_status: Result<PageListBotTaskQueryAnswer, PageListBotTaskQueryError>,
    /// The status of invidual output page.
    pub output_status: collections::BTreeMap<String, PageListBotTaskQueryOutputPageSummary>,
}

/// Struct to hold query results.
/// Error messages are formatted,
/// and titles are pretty-converted and sorted.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct PageListBotTaskQueryAnswer {
    pub titles: Vec<String>,
    pub warnings: Vec<String>,
}

/// Indicates an error a task may throw.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotTaskQueryError {
    /// The query timed out.
    /// Check your query, or adjust the timeout.
    Timeout,

    /// There were errors during query parsing.
    /// Fix the errors written in `msgs`.
    ParseError { msgs: Vec<String> },

    /// There was an error during query execution.
    /// Since the query immediately aborts upon error, there would be only at most one error.
    RuntimeError { msg: String },

    /// The query did not execute.
    /// This happens because there is no eligible page for output.
    NoQuery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotTaskQueryOutputPageSummary {
    /// The page is successfully written, either when query successful or unsuccessful (and eager or not).
    Ok,
    /// The page is skipped.
    /// This can happen when the page is found ineligible for writing, or an error occurs when checking the page.
    Skipped,
    /// The page should be written, but failed to write.
    /// This can happen when the result of the `edit` post is not successful, or when not in eager mode, the original page content failed to obtain.
    WriteError,
}
