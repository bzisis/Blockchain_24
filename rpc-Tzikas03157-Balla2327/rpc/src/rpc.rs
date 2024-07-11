use jsonrpsee::core::RpcResult;
use reth_rpc_api::RpcApiServer;
use reth_rpc_types::RpcModules;
use std::{collections::HashMap, sync::Arc};

/// `rpc` API implementation.
///
/// This type provides the functionality for handling `rpc` requests.
#[derive(Debug, Clone, Default)]
pub struct RPCApi {
    /// The implementation of the Arc API.
    rpc_modules: Arc<RpcModules>,
}

impl RPCApi {
    /// Creates a new instance of the `RPCApi` struct with the given `module_map`.
    ///
    /// # Arguments
    ///
    /// * `module_map` - A HashMap containing module names and their corresponding descriptions.
    ///
    /// # Returns
    ///
    /// A new instance of the `RPCApi` struct.
    pub fn new(module_map: HashMap<String, String>) -> Self {
        Self { rpc_modules: Arc::new(RpcModules::new(module_map)) }
    }
}

impl RpcApiServer for RPCApi {
    /// Handler for `rpc_modules`.
    ///
    /// This method returns the RPC modules available in the server.
    fn rpc_modules(&self) -> RpcResult<RpcModules> {
        Ok(self.rpc_modules.as_ref().clone())
    }
}
