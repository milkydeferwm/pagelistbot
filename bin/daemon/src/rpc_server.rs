//! Implementation of the RPC server

use std::{collections, sync::Arc};
use async_trait::async_trait;
use host::Host;
use interface::{PageListBotError, PageListBotRpcServer, rpc::NewHostConfig};
use interface::types::status::{PageListBotTaskFinderStatus, PageListBotRefresherStatus, PageListBotTaskStatus};
use tokio::sync::RwLock;
use jsonrpsee::core::RpcResult;

pub struct ServerImpl {
    host_map: Arc<RwLock<collections::HashMap<String, Host>>>,
}

impl ServerImpl {
    pub fn new(host_map: Arc<RwLock<collections::HashMap<String, Host>>>) -> Self {
        Self {
            host_map,
        }
    }
}

#[async_trait]
impl PageListBotRpcServer for ServerImpl {
    async fn new_host(&self, name: &str, config: NewHostConfig) -> RpcResult<Result<(), PageListBotError>> {
        let mut hostmap = self.host_map.write().await;
        if hostmap.contains_key(name) {
            Ok(Err(PageListBotError::HostAlreadyExists))
        } else {
            let host = Host::try_new(
                &config.username,
                &config.password,
                &config.api_endpoint,
                &config.onsite_config,
                config.prefer_bot_edit).await;
            if let Ok(host) = host {
                hostmap.insert(name.to_owned(), host);
                Ok(Ok(()))
            } else {
                Ok(Err(host.unwrap_err().into()))
            }
        }
    }

    async fn kill_host(&self, name: &str, force: bool) -> RpcResult<Result<(), PageListBotError>> {
        let tokill_host = {
            let mut hostmap = self.host_map.write().await;
            hostmap.remove(name).ok_or(PageListBotError::HostDoesNotExist)
        };
        if let Ok(host) = tokill_host {
            if !force {
                host.shutdown().await;
            }
            Ok(Ok(()))
        } else {
            Ok(Err(tokill_host.unwrap_err()))
        }
    }

    async fn scan_task_now(&self, name: &str) -> RpcResult<Result<(), PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.wakeup_finder().await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn cancel_scan_task(&self, name: &str) -> RpcResult<Result<(), PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.abort_running_finder().await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn get_finder_status(&self, name: &str) -> RpcResult<Result<PageListBotTaskFinderStatus, PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.inspect_finder().await.map_err(|e| e.into()).map(|s| s.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn refresh_api_now(&self, name: &str) -> RpcResult<Result<(), PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.wakeup_refresher().await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn cancel_refresh_api(&self, name: &str) -> RpcResult<Result<(), PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.abort_running_refresher().await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn get_refresher_status(&self, name: &str) -> RpcResult<Result<PageListBotRefresherStatus, PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.inspect_refresher().await.map_err(|e| e.into()).map(|s| s.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn run_task_now(&self, name: &str, id: u32) -> RpcResult<Result<(), PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.wakeup_task(id).await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn cancel_task(&self, name: &str, id: u32) -> RpcResult<Result<(), PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.abort_running_task(id).await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn get_task_status(&self, name: &str, id: u32) -> RpcResult<Result<PageListBotTaskStatus, PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.inspect_task(id).await.map_err(|e| e.into());
            Ok(result)
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }

    async fn get_task_list(&self, name: &str) -> RpcResult<Result<Vec<u32>, PageListBotError>> {
        let hostmap = self.host_map.read().await;
        if let Some(host) = hostmap.get(name) {
            let result = host.get_active_tasks().await;
            Ok(Ok(result))
        } else {
            Ok(Err(PageListBotError::HostDoesNotExist))
        }
    }
}
