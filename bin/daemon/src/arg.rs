//! Arguments for the daemon.

use clap::Parser;
use interface::{DEFAULT_DAEMON_ADDR, DEFAULT_DAEMON_PORT};

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub(crate) struct Arg {
    #[arg(short = 'a', long = "addr", help = "Server address to bind to.", default_value_t = DEFAULT_DAEMON_ADDR.to_string())]
    pub addr: String,

    #[arg(short = 'p', long = "port", help = "Server port to bind to.", default_value_t = DEFAULT_DAEMON_PORT)]
    pub port: u16,

    #[arg(long = "startup", help = "Read a startup file. Program will run with predefined hosts.")]
    pub startup: Option<std::path::PathBuf>,
}
