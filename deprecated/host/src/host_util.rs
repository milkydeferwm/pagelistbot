//! Utilities for `Host` shared by various submodules.

use super::Host;
use futures::{prelude::*, future};
use tokio::time;

impl Host {
    /// Utility function to parse a cron string and return the sleep future.
    pub(super) fn future_by_cron(schedule: &str) -> future::Fuse<time::Sleep> {
        use std::str::FromStr;

        let schedule = cron::Schedule::from_str(schedule);
        if let Ok(schedule) = schedule {
            let datetime = schedule.upcoming(chrono::Utc).next();
            if let Some(datetime) = datetime {
                let duration = datetime.signed_duration_since(chrono::Utc::now()).to_std();
                if let Ok(duration) = duration {
                    time::sleep(duration).fuse()
                } else {
                    // this should not happen
                    future::Fuse::terminated()
                }
            } else {
                // a one-shot cron that has expired.
                future::Fuse::terminated()
            }
        } else {
            // a problematic cron string
            future::Fuse::terminated()
        }
    }
}