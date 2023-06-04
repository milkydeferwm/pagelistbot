//! Page list bot query execution core.

#![feature(type_alias_impl_trait)]
#![feature(unix_sigpipe)]

mod api;
use api::APIDataProvider;
mod builder;
use builder::get_provider;
mod writer;
use futures::StreamExt;
use writer::*;

use ast::Expression;
use clap::Parser;
use core::time::Duration;
use intorinf::IntOrInf;
use nom::error::VerboseError;
use owo_colors::OwoColorize;
use solver::*;
use std::{
    io::{stdout, BufWriter, IsTerminal, Write},
    process::ExitCode, 
};
use trio_result::TrioResult;

#[derive(Debug, Parser)]
pub struct Arg {
    /// The user name used for query. Leave it blank if you want to execute query anonymously
    #[arg(short, long, default_value_t = String::new())]
    user: String,
    /// The password of the corresponding user. Ignored if `user` is blank.
    #[arg(short, long, default_value_t = String::new())]
    password: String,
    /// The API endpoint. This is the URL of `api.php`.
    #[arg(short, long)]
    site: String,
    /// The query string.
    #[arg(short, long)]
    query: String,
    /// Maximum time allowed for query, in seconds.
    #[arg(short, long, default_value_t = 120)]
    timeout: u64,
    /// Default maximum query result limit, if it is not overridden by `.limit()` expression modifier.
    #[arg(short, long, default_value_t = 10000)]
    limit: i32,
    /// Output in JSON format, not in human-readable format.
    #[arg(long)]
    json: bool,
}

const FAILURE_PARSE: u8 = 100;
const FAILURE_INIT: u8 = 101;
const FAILURE_SEMANTIC: u8 = 102;
const FAILURE_QUERY: u8 = 103;

#[tokio::main]
#[unix_sigpipe = "sig_dfl"]
async fn main() -> ExitCode {
    let arg = Arg::parse();
    let stdout = stdout().lock();
    let color = stdout.is_terminal();
    let mut writer = BufWriter::new(stdout);

    // parse the expression first. only continue if parse successful.
    let expr = match Expression::parse::<VerboseError<_>>(&arg.query) {
        Ok(expr) => expr,
        Err(e) => {
            write_err(e, writer.get_mut(), color, arg.json).unwrap();
            return ExitCode::from(FAILURE_PARSE);
        }
    };

    // login, set up data provider.
    let provider = match get_provider(&arg.site, &arg.user, &arg.password).await {
        Ok(provider) => provider,
        Err(e) => {
            write_err(e, writer.get_mut(), color, arg.json).unwrap();
            return ExitCode::from(FAILURE_INIT);
        }
    };

    // set up stream.
    let mut stream = match SolverStream::from_expr(&expr, provider.clone(), IntOrInf::from(arg.limit)) {
        Ok(stream) => stream,
        Err(e) => {
            write_err(e, writer.get_mut(), color, arg.json).unwrap();
            return ExitCode::from(FAILURE_SEMANTIC);
        }
    };

    // perform query.
    let sleep = tokio::time::sleep(Duration::from_secs(arg.timeout));
    tokio::pin!(sleep);

    let mut item_count = 0;
    let mut warn_count = 0;

    loop {
        tokio::select! {
            biased;
            _ = &mut sleep => {
                // time elapsed.
                warn_count += 1;
                write_warn(format_args!("timeout after {} seconds", arg.timeout), writer.get_mut(), color, arg.json).unwrap();
                break;
            },
            item = stream.next() => {
                if let Some(item) = item {
                    match item {
                        TrioResult::Ok(item) => {
                            let t = match item.get_title() {
                                Ok(t) => t,
                                Err(e) => {
                                    write_err(e, writer.get_mut(), color, arg.json).unwrap();
                                    return ExitCode::from(FAILURE_QUERY);
                                },
                            };
                            item_count += 1;
                            write_item(provider.title_codec.to_pretty(t), writer.get_mut(), arg.json).unwrap();
                        },
                        TrioResult::Warn(w) => {
                            warn_count += 1;
                            write_warn(w, writer.get_mut(), color, arg.json).unwrap();
                        },
                        TrioResult::Err(e) => {
                            write_err(e, writer.get_mut(), color, arg.json).unwrap();
                            return ExitCode::from(FAILURE_QUERY);
                        },
                    }
                } else {
                    // poll finished.
                    break;
                }
            }
        }
    }
    
    // write summary
    if !arg.json && color {
        writeln!(writer, "{}", format_args!("total: {item_count}, warning: {warn_count}").bold()).unwrap();
    }
    ExitCode::SUCCESS
}