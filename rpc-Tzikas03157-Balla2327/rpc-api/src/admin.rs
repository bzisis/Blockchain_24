use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_network_peers::{AnyNode, NodeRecord};
use reth_rpc_types::admin::{NodeInfo, PeerInfo};

/// Admin namespace rpc interface that gives access to several non-standard RPC methods.
#[cfg_attr(not(feature = "client"), rpc(server, namespace = "admin"))]
#[cfg_attr(feature = "client", rpc(server, client, namespace = "admin"))]
pub trait AdminApi {
    /// Adds the given node record to the peerset.
    ///
    /// # Arguments
    ///
    /// * `record` - The `NodeRecord` representing the node to be added.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing a boolean indicating success.
    #[method(name = "addPeer")]
    fn add_peer(&self, record: NodeRecord) -> RpcResult<bool>;

    /// Disconnects from a remote node if the connection exists.
    ///
    /// # Arguments
    ///
    /// * `record` - The `AnyNode` representing the node to be removed.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing a boolean indicating success.
    #[method(name = "removePeer")]
    fn remove_peer(&self, record: AnyNode) -> RpcResult<bool>;

    /// Adds the given node record to the trusted peerset.
    ///
    /// # Arguments
    ///
    /// * `record` - The `AnyNode` representing the node to be added as trusted.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing a boolean indicating success.
    #[method(name = "addTrustedPeer")]
    fn add_trusted_peer(&self, record: AnyNode) -> RpcResult<bool>;

    /// Removes a remote node from the trusted peer set, but it does not disconnect it
    /// automatically.
    ///
    /// # Arguments
    ///
    /// * `record` - The `AnyNode` representing the node to be removed from trusted peers.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing a boolean indicating success.
    #[method(name = "removeTrustedPeer")]
    fn remove_trusted_peer(&self, record: AnyNode) -> RpcResult<bool>;

    /// The peers administrative property can be queried for all the information known about the
    /// connected remote nodes at the networking granularity. These include general information
    /// about the nodes themselves as participants of the devp2p P2P overlay protocol, as well as
    /// specialized information added by each of the running application protocols.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing a vector of `PeerInfo`.
    #[method(name = "peers")]
    async fn peers(&self) -> RpcResult<Vec<PeerInfo>>;

    /// Creates an RPC subscription which serves events received from the network.
    ///
    /// # Returns
    ///
    /// A `SubscriptionResult` for peer events.
    #[subscription(
        name = "peerEvents",
        unsubscribe = "peerEvents_unsubscribe",
        item = String
    )]
    async fn subscribe_peer_events(&self) -> jsonrpsee::core::SubscriptionResult;

    /// Returns the ENR (Ethereum Node Record) of the node.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing the `NodeInfo`.
    #[method(name = "nodeInfo")]
    async fn node_info(&self) -> RpcResult<NodeInfo>;
}
