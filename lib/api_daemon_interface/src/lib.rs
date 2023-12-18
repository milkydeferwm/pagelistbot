use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde_json::Value;
use std::collections::HashMap;

/// The service interface the API Backend Service provides.
#[rpc(server, client)]
pub trait APIServiceInterface {
    /// Retrieve the site info. This can be used to build a `mwtitle::TitleCodec`.
    #[method(name = "getSiteInfo")]
    async fn get_site_info(&self, key: &str) -> RpcResult<Value>;

    /// Retrieve the `apihighlimits` flag status.
    #[method(name = "getApiHighLimits")]
    async fn get_apihighlimits(&self, key: &str) -> RpcResult<bool>;

    /// Retrieve the `bot` flag status.
    #[method(name = "getBot")]
    async fn get_bot(&self, key: &str) -> RpcResult<bool>;

    /// Send a query by GET.
    #[method(name = "getValue")]
    async fn get_value(&self, key: &str, parameters: HashMap<String, String>) -> RpcResult<Value>;

    /// Send a query by POST.
    #[method(name = "postValue")]
    async fn post_value(&self, key: &str, parameters: HashMap<String, String>) -> RpcResult<Value>;

    /// Send a query by POST with token.
    #[method(name = "postValueWithToken")]
    async fn post_value_with_token(&self, key: &str, token_type: &str, parameters: HashMap<String, String>) -> RpcResult<Value>;
}
