//! Pool component for the node builder.

use crate::{BuilderContext, FullNodeTypes};
use reth_transaction_pool::TransactionPool;
use std::future::Future;

/// A trait for building the transaction pool for a node.
///
/// This trait defines the method for creating the transaction pool.
pub trait PoolBuilder<Node: FullNodeTypes>: Send {
    /// The type of the transaction pool to build.
    type Pool: TransactionPool + Unpin + 'static;

    /// Creates the transaction pool.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The builder context.
    ///
    /// # Returns
    ///
    /// A future that resolves to a `Result` containing the transaction pool.
    fn build_pool(
        self,
        ctx: &BuilderContext<Node>,
    ) -> impl Future<Output = eyre::Result<Self::Pool>> + Send;
}

impl<Node, F, Fut, Pool> PoolBuilder<Node> for F
where
    Node: FullNodeTypes,
    Pool: TransactionPool + Unpin + 'static,
    F: FnOnce(&BuilderContext<Node>) -> Fut + Send,
    Fut: Future<Output = eyre::Result<Pool>> + Send,
{
    type Pool = Pool;

    /// Creates the transaction pool.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The builder context.
    ///
    /// # Returns
    ///
    /// A future that resolves to a `Result` containing the transaction pool.
    fn build_pool(
        self,
        ctx: &BuilderContext<Node>,
    ) -> impl Future<Output = eyre::Result<Self::Pool>> {
        self(ctx)
    }
}