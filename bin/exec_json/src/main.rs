//! Execute a query under on-site JSON specification.
//! In fact the JSON file is downloaded and cached locally.
//! The local cache is passed as input file.

use clap::Parser;
use std::{env, process::ExitCode};

struct Arg {
    
}

#[tokio::main]
async fn main() -> ExitCode {
    ExitCode::SUCCESS
}
