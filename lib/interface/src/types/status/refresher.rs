//! Interface types for API refresher.

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
pub struct PageListBotRefresherStatus {
    pub last_run_status: PageListBotRefresherRoutineStatus,
    #[cfg_attr(feature="use_serde", serde_as(as = "DisplayFromStr"))]
    pub last_run_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotRefresherRoutineStatus {
    /// No refresher process was run yet
    NoRun,
    /// The refresher process was running
    Running,
    /// The refresher process was forcefully aborted.
    Aborted,
    /// The refresher process was finished, with corresponding results.
    Finished(PageListBotRefresherSummary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="use_serde", derive(Serialize, Deserialize))]
pub enum PageListBotRefresherSummary {
    /// The current API client is validated.
    Validated,
    /// The current API client is refreshed.
    Refreshed,
    /// Failed to build a new API client.
    NewClientFailed(String),
    /// Failed to grab new site info.
    NewSiteInfoFailed(String),
    /// Failed to grab new user info.
    NewUserInfoFailed(String),
}
