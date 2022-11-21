//! Arguments for the daemon.

use clap::{Arg, ArgAction, Command, command, value_parser};
use interface::{DEFAULT_DAEMON_ADDR, DEFAULT_DAEMON_PORT};

pub(crate) fn build_args() -> Command {
    command!()
        .args([
            Arg::new("addr")
                .default_value(DEFAULT_DAEMON_ADDR)
                .long("addr")
                .short('a')
                .help("Server address to bind to."),
            Arg::new("port")
                .default_value(DEFAULT_DAEMON_PORT)
                .long("port")
                .short('p')
                .value_parser(value_parser!(u16))
                .help("Server port to bind to."),
            Arg::new("skip")
                .long("skip-startup")
                .help("Skips reading the startup file. Program will run without any host.")
                .action(ArgAction::SetTrue),
        ])
}
