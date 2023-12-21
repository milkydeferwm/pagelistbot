//! JSON RPC handler.
//! This module provides functionality for RPC calls.
//! It defines the interface, which is also visible to other crates.

use crate::APIConnection;
use jsonrpsee::core::RpcResult;
use pagelistbot_api_daemon_interface::APIServiceInterfaceServer;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// All possible errors emitted by API Backend Service.
#[derive(Debug, thiserror::Error)]
enum APIServiceError {
    #[error("no connection for `{0}`")]
    NoConnection(String),

    #[error(transparent)]
    MwApi(#[from] mwapi::Error),
}

impl APIServiceError {
    #[inline]
    fn code(&self) -> i32 {
        match self {
            Self::NoConnection(_) => 10000,
            Self::MwApi(_) => 10001,
        }
    }

    #[inline]
    fn data(&self) -> Option<Value> {
        None
    }
}

impl From<APIServiceError> for jsonrpsee::types::ErrorObjectOwned {
    fn from(value: APIServiceError) -> Self {
        Self::owned(value.code(), value.to_string(), value.data())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct APIServiceImpl {
    store: Arc<RwLock<HashMap<String, APIConnection>>>,
}

impl APIServiceImpl {
    pub fn new(store: Arc<RwLock<HashMap<String, APIConnection>>>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl APIServiceInterfaceServer for APIServiceImpl {
    async fn get_site_info(&self, key: &str) -> RpcResult<Value> {
        let store = self.store.read().await;
        let connection = store.get(key).ok_or(APIServiceError::NoConnection(key.into()))?;
        Ok(connection.site_info.to_owned())
    }

    async fn get_apihighlimits(&self, key: &str) -> RpcResult<bool> {
        let store = self.store.read().await;
        let connection = store.get(key).ok_or(APIServiceError::NoConnection(key.into()))?;
        Ok(connection.apihighlimits)
    }

    async fn get_bot(&self, key: &str) -> RpcResult<bool> {
        let store = self.store.read().await;
        let connection = store.get(key).ok_or(APIServiceError::NoConnection(key.into()))?;
        Ok(connection.bot)
    }

    async fn get_value(&self, key: &str, parameters: HashMap<String, String>) -> RpcResult<Value> {
        let store = self.store.read().await;
        let connection = store.get(key).ok_or(APIServiceError::NoConnection(key.into()))?;
        let ret = connection.client.get_value(parameters).await.map_err(APIServiceError::from)?;
        Ok(ret)
    }

    async fn post_value(&self, key: &str, parameters: HashMap<String, String>) -> RpcResult<Value> {
        let store = self.store.read().await;
        let connection = store.get(key).ok_or(APIServiceError::NoConnection(key.into()))?;
        let ret = connection.client.post_value(parameters).await.map_err(APIServiceError::from)?;
        Ok(ret)
    }

    async fn post_value_with_token(&self, key: &str, token_type: &str, parameters: HashMap<String, String>) -> RpcResult<Value> {
        let store = self.store.read().await;
        let connection = store.get(key).ok_or(APIServiceError::NoConnection(key.into()))?;
        let ret = connection.client.post_with_token(token_type, parameters).await.map_err(APIServiceError::from)?;
        Ok(ret)
    }
}
