//! RPC interface.

#![cfg(feature = "rpc")]

pub use crate::types::rpc::NewHostConfig;
use crate::types::status::{PageListBotTaskFinderStatus, PageListBotRefresherStatus, PageListBotTaskStatus};
use crate::error::PageListBotError;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[rpc(client, server)]
pub trait PageListBotRpc {
    /// Create a new `Host`.
    /// 
    /// The new `Host` is assigned a key (`name`) and is used to refer to this host.
    /// If there is already another host by the same name, the method will fail with `HostAlreadyExists`.
    #[method(name = "new_host")]
    async fn new_host(&self, name: &str, config: NewHostConfig) -> RpcResult<Result<(), PageListBotError>>;

    /// Kill an existing `Host`.
    /// 
    /// The host is identified by its `name`.
    /// The `force` options allows to directly drop the `Host` without shutdown procedure.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    #[method(name = "kill_host")]
    async fn kill_host(&self, name: &str, force: bool) -> RpcResult<Result<(), PageListBotError>>;

    /// Ask the host to scan for tasks now.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "scan_task_now")]
    async fn scan_task_now(&self, name: &str) -> RpcResult<Result<(), PageListBotError>>;

    /// Ask the host to stop the task finder routine.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "cancel_scan_task")]
    async fn cancel_scan_task(&self, name: &str) -> RpcResult<Result<(), PageListBotError>>;

    /// Ask the host for task finder routine's status.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "get_finder_status")]
    async fn get_finder_status(&self, name: &str) -> RpcResult<Result<PageListBotTaskFinderStatus, PageListBotError>>;

    /// Ask the host to refresh the API now.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "refresh_api_now")]
    async fn refresh_api_now(&self, name: &str) -> RpcResult<Result<(), PageListBotError>>;

    // Ask the host to stop the API refresher routine.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "cancel_refresh_api")]
    async fn cancel_refresh_api(&self, name: &str) -> RpcResult<Result<(), PageListBotError>>;

    /// Ask the host for API refresher routine's status.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "get_refresher_status")]
    async fn get_refresher_status(&self, name: &str) -> RpcResult<Result<PageListBotRefresherStatus, PageListBotError>>;

    /// Ask the host to activate the task routine now.
    /// 
    /// The host is identified by its `name`, and the task is identified by its "taskid" aka page id.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "run_task_now")]
    async fn run_task_now(&self, name: &str, id: u32) -> RpcResult<Result<(), PageListBotError>>;

    // Ask the host to stop the task routine.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "cancel_task")]
    async fn cancel_task(&self, name: &str, id: u32) -> RpcResult<Result<(), PageListBotError>>;

    /// Ask the host for the task routine's status.
    /// 
    /// The host is identified by its `name`, and the task is identified by its "taskid" aka page id.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "get_task_status")]
    async fn get_task_status(&self, name: &str, id: u32) -> RpcResult<Result<PageListBotTaskStatus, PageListBotError>>;

    /// Ask the host for a list of running task routines.
    /// 
    /// The host is identified by its `name`.
    /// If there is no host by the `name`, the method will fail with `HostDoesNotExist`.
    /// If there are other kind of errors, the method will fail with `HostError`.
    #[method(name = "get_task_list")]
    async fn get_task_list(&self, name: &str) -> RpcResult<Result<Vec<u32>, PageListBotError>>;
}
