use clap::{Parser, Subcommand};
use interface::{DEFAULT_DAEMON_ADDR, DEFAULT_DAEMON_PORT};

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(version, about)]
pub(crate) struct Arg {
    #[arg(short = 'a', long = "addr", default_value_t = DEFAULT_DAEMON_ADDR.to_string())]
    pub addr: String,
    #[arg(short = 'p', long = "port", default_value_t = DEFAULT_DAEMON_PORT)]
    pub port: u16,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Commands {
    NewHost {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
        #[arg(short, long)]
        api_endpoint: String,
        #[arg(short, long)]
        db: Option<String>,
        #[arg(short, long)]
        onsite_config: String,
        #[arg(short, long)]
        bot: bool,
    },
    ListHost,
    KillHost {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    ScanTask {
        #[arg(short, long)]
        name: String
    },
    StopScanTask {
        #[arg(short, long)]
        name: String
    },
    GetFinderStatus {
        #[arg(short, long)]
        name: String
    },
    RefreshAPI {
        #[arg(short, long)]
        name: String
    },
    StopRefreshAPI {
        #[arg(short, long)]
        name: String
    },
    GetRefresherStatus {
        #[arg(short, long)]
        name: String
    },
    RunTask {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        id: u32,
    },
    ListTask {
        #[arg(short, long)]
        name: String,
    },
    StopRunTask {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        id: u32,
    },
    GetTaskStatus {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        id: u32,
    }
}
