use crate::rpc::{RethRpcServerHandles, RpcRegistry};
use reth_network::NetworkHandle;
use reth_node_api::FullNodeComponents;
use reth_node_core::{
    dirs::{ChainPath, DataDirPath},
    node_config::NodeConfig,
    rpc::{
        api::EngineApiClient,
        builder::{auth::AuthServerHandle, RpcServerHandle},
    },
};
use reth_payload_builder::PayloadBuilderHandle;
use reth_primitives::ChainSpec;
use reth_provider::ChainSpecProvider;
use reth_tasks::TaskExecutor;
use std::sync::Arc;

// Re-export the node API types
use crate::components::NodeComponentsBuilder;
pub use reth_node_api::{FullNodeTypes, NodeTypes};

/// A [crate::Node] is a [NodeTypes] that comes with preconfigured components.
///
/// This can be used to configure the builder with a preset of components.
///
/// # Type Parameters
///
/// - `N`: A type that implements the [`FullNodeTypes`] trait.
pub trait Node<N: FullNodeTypes>: NodeTypes + Clone {
    /// The type that builds the node's components.
    type ComponentsBuilder: NodeComponentsBuilder<N>;

    /// Returns a [NodeComponentsBuilder] for the node.
    fn components_builder(self) -> Self::ComponentsBuilder;
}

/// The launched node with all components including RPC handlers.
///
/// This struct provides access to all components of the launched node,
/// including the EVM configuration, transaction pool, network handle,
/// provider, payload builder, task executor, and RPC server handles.
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
#[derive(Debug)]
pub struct FullNode<Node: FullNodeComponents> {
    /// The EVM configuration.
    pub evm_config: Node::Evm,
    /// The node's transaction pool.
    pub pool: Node::Pool,
    /// Handle to the node's network.
    pub network: NetworkHandle,
    /// Provider to interact with the node's database.
    pub provider: Node::Provider,
    /// Handle to the node's payload builder service.
    pub payload_builder: PayloadBuilderHandle<Node::Engine>,
    /// Task executor for the node.
    pub task_executor: TaskExecutor,
    /// Handles to the node's RPC servers.
    pub rpc_server_handles: RethRpcServerHandles,
    /// The configured RPC namespaces.
    pub rpc_registry: RpcRegistry<Node>,
    /// The initial node configuration.
    pub config: NodeConfig,
    /// The data directory of the node.
    pub data_dir: ChainPath<DataDirPath>,
}

impl<Node: FullNodeComponents> FullNode<Node> {
    /// Returns the [ChainSpec] of the node.
    ///
    /// The ChainSpec provides information about the chain's configuration and parameters.
    ///
    /// # Returns
    ///
    /// An `Arc` containing the [ChainSpec].
    pub fn chain_spec(&self) -> Arc<ChainSpec> {
        self.provider.chain_spec()
    }

    /// Returns the [RpcServerHandle] to the started RPC server.
    ///
    /// The RpcServerHandle provides access to the running RPC server, allowing
    /// for interactions such as stopping the server.
    ///
    /// # Returns
    ///
    /// A reference to the [RpcServerHandle].
    pub fn rpc_server_handle(&self) -> &RpcServerHandle {
        &self.rpc_server_handles.rpc
    }

    /// Returns the [AuthServerHandle] to the started authenticated engine API server.
    ///
    /// The AuthServerHandle provides access to the authenticated engine API server,
    /// allowing for interactions such as stopping the server.
    ///
    /// # Returns
    ///
    /// A reference to the [AuthServerHandle].
    pub fn auth_server_handle(&self) -> &AuthServerHandle {
        &self.rpc_server_handles.auth
    }

    /// Returns the [EngineApiClient] interface for the authenticated engine API.
    ///
    /// This will send authenticated HTTP requests to the node's auth server.
    ///
    /// # Returns
    ///
    /// An instance of the [EngineApiClient] for HTTP communication.
    pub fn engine_http_client(&self) -> impl EngineApiClient<Node::Engine> {
        self.auth_server_handle().http_client()
    }

    /// Returns the [EngineApiClient] interface for the authenticated engine API.
    ///
    /// This will send authenticated WebSocket requests to the node's auth server.
    ///
    /// # Returns
    ///
    /// An instance of the [EngineApiClient] for WebSocket communication.
    pub async fn engine_ws_client(&self) -> impl EngineApiClient<Node::Engine> {
        self.auth_server_handle().ws_client().await
    }

    /// Returns the [EngineApiClient] interface for the authenticated engine API.
    ///
    /// This will send authenticated IPC requests to the node's auth server.
    ///
    /// # Returns
    ///
    /// An `Option` containing an instance of the [EngineApiClient] for IPC communication,
    /// if the platform supports IPC.
    #[cfg(unix)]
    pub async fn engine_ipc_client(&self) -> Option<impl EngineApiClient<Node::Engine>> {
        self.auth_server_handle().ipc_client().await
    }
}

impl<Node: FullNodeComponents> Clone for FullNode<Node> {
    /// Creates a clone of the `FullNode`.
    ///
    /// This method clones all components of the `FullNode`, including the EVM configuration,
    /// transaction pool, network handle, provider, payload builder, task executor,
    /// RPC server handles, RPC registry, initial configuration, and data directory.
    ///
    /// # Returns
    ///
    /// A new instance of `FullNode` with cloned components.
    fn clone(&self) -> Self {
        Self {
            evm_config: self.evm_config.clone(),
            pool: self.pool.clone(),
            network: self.network.clone(),
            provider: self.provider.clone(),
            payload_builder: self.payload_builder.clone(),
            task_executor: self.task_executor.clone(),
            rpc_server_handles: self.rpc_server_handles.clone(),
            rpc_registry: self.rpc_registry.clone(),
            config: self.config.clone(),
            data_dir: self.data_dir.clone(),
        }
    }
}