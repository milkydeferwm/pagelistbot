//! Execute a query under on-site JSON specification.
//! In fact the JSON file is downloaded and cached locally.
//! The local cache is passed as input file.

#![allow(dead_code)]
#![allow(unused_imports)]

use clap::Parser;
use std::{env, process::ExitCode};

struct Arg {
    
}

#[tokio::main]
async fn main() -> ExitCode {
    ExitCode::SUCCESS
}
