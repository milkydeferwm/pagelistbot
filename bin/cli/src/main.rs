mod arg;
mod error;

use arg::Commands;
use clap::Parser;
use error::ClientError;
use interface::PageListBotRpcClient;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let args = arg::Arg::parse();
    let addr = args.addr;
    let port = args.port;
    let client = HttpClientBuilder::default().build(format!("http://{}:{}", addr, port))?;

    match args.command {
        Commands::NewHost { name, username, password, api_endpoint, db, onsite_config, bot } => handle_new_host(&client, &name, &username, &password, &api_endpoint, &db, &onsite_config, bot).await,
        Commands::ListHost => handle_list_host(&client).await,
        Commands::KillHost { name, force } => handle_kill_host(&client, &name, force).await,
        Commands::ScanTask { name } => handle_scan_task(&client, &name).await,
        Commands::StopScanTask { name } => handle_stop_scan_task(&client, &name).await,
        Commands::GetFinderStatus { name } => handle_get_finder_status(&client, &name).await,
        Commands::RefreshAPI { name } => handle_refresh_api(&client, &name).await,
        Commands::StopRefreshAPI { name } => handle_stop_refresh_api(&client, &name).await,
        Commands::GetRefresherStatus { name } => handle_get_refresher_status(&client, &name).await,
        Commands::RunTask { name, id } => handle_run_task(&client, &name, id).await,
        Commands::ListTask { name } => handle_list_task(&client, &name).await,
        Commands::StopRunTask { name, id } => handle_stop_run_task(&client, &name, id).await,
        Commands::GetTaskStatus { name, id } => handle_get_task_status(&client, &name, id).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_new_host(client: &HttpClient, name: &str, username: &str, password: &str, api_endpoint: &str, db: &Option<String>, onsite_config: &str, bot: bool) -> Result<(), ClientError> {
    let cfg = interface::rpc::NewHostConfig {
        api_endpoint: api_endpoint.to_owned(),
        username: username.to_owned(),
        password: password.to_owned(),
        onsite_config: onsite_config.to_owned(),
        prefer_bot_edit: bot,
        db_name: db.to_owned(),
    };
    client.new_host(name, cfg).await??;
    println!("Create host `{}` successful.", name);
    Ok(())
}

async fn handle_list_host(client: &HttpClient) -> Result<(), ClientError> {
    let list = client.get_host_list().await?;
    println!("Running hosts:");
    for h in list {
        println!("\t{}", h);
    }
    Ok(())
}

async fn handle_kill_host(client: &HttpClient, name: &str, force: bool) -> Result<(), ClientError> {
    client.kill_host(name, force).await??;
    println!("Kill host `{}` successful.", name);
    Ok(())
}

async fn handle_scan_task(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    client.scan_task_now(name).await??;
    println!("Host `{}` is now scanning tasks.", name);
    Ok(())
}

async fn handle_stop_scan_task(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    client.cancel_scan_task(name).await??;
    println!("Host `{}` is now stopped scanning tasks.", name);
    Ok(())
}

async fn handle_get_finder_status(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    use interface::types::status::finder::{PageListBotTaskFinderRoutineStatus, PageListBotTaskFinderSummary, PageListBotTaskChange};
    let status = client.get_finder_status(name).await??;
    println!("Task finder for host `{}`:", name);
    if status.last_run_status == PageListBotTaskFinderRoutineStatus::NoRun {
        println!("\tLast scan: never");
    } else {
        println!("\tLast scan: {}", status.last_run_time);
    }
    match status.last_run_status {
        PageListBotTaskFinderRoutineStatus::NoRun => println!("\tStatus: not scanned yet"),
        PageListBotTaskFinderRoutineStatus::Running => println!("\tStatus: now scanning"),
        PageListBotTaskFinderRoutineStatus::Aborted => println!("\tStatus: last scan cancelled"),
        PageListBotTaskFinderRoutineStatus::Finished(s) => {
            match s {
                PageListBotTaskFinderSummary::GlobalConfigFailed(e) => println!("\tStatus: cannot update global configuration: {}", e),
                PageListBotTaskFinderSummary::TaskListFailed(e) => println!("\tStatus: cannot fetch task list: {}", e),
                PageListBotTaskFinderSummary::Success(m) => {
                    fn filter_print_or_none(changemap: &std::collections::HashMap<u32, PageListBotTaskChange>, filter_target: PageListBotTaskChange) {
                        let mut ls = changemap.iter()
                            .filter(|(_, &c)| c == filter_target)
                            .map(|(&id, _)| id)
                            .collect::<Vec<_>>();
                        ls.sort();
                        if ls.is_empty() {
                            println!("\t\t\t<none>");
                        } else {
                            for l in ls {
                                println!("\t\t\t{}", l);
                            }
                        }
                    }
                    println!("\tStatus: scan successful");
                    println!("\t\tThe following tasks are added:");
                    filter_print_or_none(&m, PageListBotTaskChange::Created);
                    println!("\t\tThe following tasks are updated:");
                    filter_print_or_none(&m, PageListBotTaskChange::Updated);
                    println!("\t\tThe following tasks are removed:");
                    filter_print_or_none(&m, PageListBotTaskChange::Killed);
                    println!("\t\tThe following tasks are restarted:");
                    filter_print_or_none(&m, PageListBotTaskChange::Restarted);
                    println!("\t\tThe following tasks are skipped:");
                    filter_print_or_none(&m, PageListBotTaskChange::Skipped);
                    println!("\t\tAll other tasks remain unchanged.")
                }
            }
        },
    }
    Ok(())
}

async fn handle_refresh_api(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    client.refresh_api_now(name).await??;
    println!("Host `{}` is now refreshing.", name);
    Ok(())
}

async fn handle_stop_refresh_api(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    client.cancel_refresh_api(name).await??;
    println!("Host `{}` is now stopped refreshing.", name);
    Ok(())
}

async fn handle_get_refresher_status(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    use interface::types::status::refresher::{PageListBotRefresherRoutineStatus, PageListBotRefresherSummary};
    let status = client.get_refresher_status(name).await??;
    println!("API refresher for host `{}`:", name);
    if status.last_run_status == PageListBotRefresherRoutineStatus::NoRun {
        println!("\tLast refresh: never");
    } else {
        println!("\tLast refresh: {}", status.last_run_time);
    }
    match status.last_run_status {
        PageListBotRefresherRoutineStatus::NoRun => println!("\tStatus: never refreshed yet"),
        PageListBotRefresherRoutineStatus::Running => println!("\tStatus: now refreshing"),
        PageListBotRefresherRoutineStatus::Aborted => println!("\tStatus: last refresh cancelled"),
        PageListBotRefresherRoutineStatus::Finished(s) => {
            match s {
                PageListBotRefresherSummary::Validated => println!("\tStatus: API is validated"),
                PageListBotRefresherSummary::Refreshed => println!("\tStatus: API is refreshed"),
                PageListBotRefresherSummary::NewClientFailed(e) => println!("\tStatus: Cannot build new API client: {}", e),
                PageListBotRefresherSummary::NewSiteInfoFailed(e) => println!("\tStatus: Cannot fetch site information: {}", e),
                PageListBotRefresherSummary::NewUserInfoFailed(e) => println!("\tStatus: Cannot fetch user information: {}", e),
            }
        }
    }
    Ok(())
}

async fn handle_run_task(client: &HttpClient, name: &str, id: u32) -> Result<(), ClientError> {
    client.run_task_now(name, id).await??;
    println!("Task `{}` on host `{}` is now running.", id, name);
    Ok(())
}

async fn handle_list_task(client: &HttpClient, name: &str) -> Result<(), ClientError> {
    let list = client.get_task_list(name).await??;
    println!("Running tasks on host `{}`:", name);
    for h in list {
        println!("\t{}", h);
    }
    Ok(())
}

async fn handle_stop_run_task(client: &HttpClient, name: &str, id: u32) -> Result<(), ClientError> {
    client.cancel_task(name, id).await??;
    println!("Task `{}` on host `{}` is now stopped until next wakeup.", id, name);
    println!("To disable the task, edit the task description page.");
    Ok(())
}

async fn handle_get_task_status(client: &HttpClient, name: &str, id: u32) -> Result<(), ClientError> {
    use interface::types::status::task::{PageListBotTaskQueryStatus, PageListBotTaskQueryError, PageListBotTaskQueryOutputPageSummary};
    let status = client.get_task_status(name, id).await??;
    println!("Task `{}` on host `{}`:", id, name);
    if status.last_run_status == PageListBotTaskQueryStatus::NoRun {
        println!("\tLast run: never");
    } else {
        println!("\tLast run: {}", status.last_run_time);
    }
    match status.last_run_status {
        PageListBotTaskQueryStatus::NoRun => println!("\tStatus: never executed yet"),
        PageListBotTaskQueryStatus::Running => println!("\tStatus: now running"),
        PageListBotTaskQueryStatus::Aborted => println!("\tStatus: last run cancelled"),
        PageListBotTaskQueryStatus::Finished(s) => {
            println!("\tStatus: last run successful");
            match s.query_status {
                Ok(ans) => {
                    println!("\t\tQuery: successful with {} result(s) and {} warning(s)", ans.titles.len(), ans.warnings.len());
                    if !ans.warnings.is_empty() {
                        println!("\t\tWarnings:");
                        for w in ans.warnings.iter() {
                            println!("\t\t\t{}", w);
                        }
                    }
                },
                Err(err) => {
                    match err {
                        PageListBotTaskQueryError::Timeout => println!("\t\tQuery: time out"),
                        PageListBotTaskQueryError::ParseError { msgs } => {
                            println!("\t\tQuery: cannot parse expression");
                            println!("\t\tErrors:");
                            for e in msgs.iter() {
                                println!("\t\t\t{}", e);
                            }
                        },
                        PageListBotTaskQueryError::RuntimeError { msg } => {
                            println!("\t\tQuery: execution error");
                            println!("\t\tError: {}", msg);
                        },
                        PageListBotTaskQueryError::NoQuery => println!("\t\tQuery: not conducted, no output page or all skipped"),
                    }
                }
            }
            println!("\t\tOutput page status:");
            for (p, r) in s.output_status.iter() {
                match r {
                    PageListBotTaskQueryOutputPageSummary::Ok => println!("\t\t\t{} -> success", p),
                    PageListBotTaskQueryOutputPageSummary::Skipped => println!("\t\t\t{} -> skip", p),
                    PageListBotTaskQueryOutputPageSummary::WriteError => println!("\t\t\t{} -> edit error", p),
                }
            }
            println!("\t\tRefer to log for detailed information.")
        },
    }
    Ok(())
}
