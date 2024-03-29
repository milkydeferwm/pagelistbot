//! Environment utilities for Page List Bot.

use std::{env, path::PathBuf};

#[inline]
pub fn pagelistbot_home() -> PathBuf {
    if let Ok(x) = env::var("PAGELISTBOT_HOME") {
        PathBuf::from(x)
    } else {
        home::home_dir().unwrap_or(PathBuf::from("~")).join(".pagelistbot")
    }
}

#[inline]
pub fn pagelistbot_bin() -> PathBuf {
    pagelistbot_home().join("bin")
}

#[inline]
pub fn pagelistbot_log() -> PathBuf {
    pagelistbot_home().join("logs")
}
