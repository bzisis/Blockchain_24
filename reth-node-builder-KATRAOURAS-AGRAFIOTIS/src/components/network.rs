//! Network component for the node builder.

use crate::{BuilderContext, FullNodeTypes};
use reth_network::NetworkHandle;
use reth_transaction_pool::TransactionPool;
use std::future::Future;

/// A trait for building the network implementation for a node.
///
/// This trait defines the method for launching the network implementation and returning a handle to it.
pub trait NetworkBuilder<Node: FullNodeTypes, Pool: TransactionPool>: Send {
    /// Launches the network implementation and returns the handle to it.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The builder context.
    /// - `pool`: The transaction pool.
    ///
    /// # Returns
    ///
    /// A future that resolves to a `Result` containing the `NetworkHandle`.
    fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> impl Future<Output = eyre::Result<NetworkHandle>> + Send;
}

impl<Node, F, Fut, Pool> NetworkBuilder<Node, Pool> for F
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
    F: Fn(&BuilderContext<Node>, Pool) -> Fut + Send,
    Fut: Future<Output = eyre::Result<NetworkHandle>> + Send,
{
    /// Launches the network implementation and returns the handle to it.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The builder context.
    /// - `pool`: The transaction pool.
    ///
    /// # Returns
    ///
    /// A future that resolves to a `Result` containing the `NetworkHandle`.
    fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> impl Future<Output = eyre::Result<NetworkHandle>> + Send {
        self(ctx, pool)
    }
}