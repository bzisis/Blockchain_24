//! Payload service component for the node builder.

use crate::{BuilderContext, FullNodeTypes};
use reth_payload_builder::PayloadBuilderHandle;
use reth_transaction_pool::TransactionPool;
use std::future::Future;

/// A trait for spawning the payload service for a node.
///
/// This trait defines the method for launching the payload service and returning a handle to it.
pub trait PayloadServiceBuilder<Node: FullNodeTypes, Pool: TransactionPool>: Send {
    /// Spawns the payload service and returns the handle to it.
    ///
    /// The [`BuilderContext`] is provided to allow access to the node's configuration.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The builder context.
    /// - `pool`: The transaction pool.
    ///
    /// # Returns
    ///
    /// A future that resolves to a `Result` containing the [`PayloadBuilderHandle`].
    fn spawn_payload_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> impl Future<Output = eyre::Result<PayloadBuilderHandle<Node::Engine>>> + Send;
}

impl<Node, F, Fut, Pool> PayloadServiceBuilder<Node, Pool> for F
where
    Node: FullNodeTypes,
    Pool: TransactionPool,
    F: Fn(&BuilderContext<Node>, Pool) -> Fut + Send,
    Fut: Future<Output = eyre::Result<PayloadBuilderHandle<Node::Engine>>> + Send,
{
    /// Spawns the payload service and returns the handle to it.
    ///
    /// The [`BuilderContext`] is provided to allow access to the node's configuration.
    ///
    /// # Arguments
    ///
    /// - `ctx`: The builder context.
    /// - `pool`: The transaction pool.
    ///
    /// # Returns
    ///
    /// A future that resolves to a `Result` containing the [`PayloadBuilderHandle`].
    fn spawn_payload_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> impl Future<Output = eyre::Result<PayloadBuilderHandle<Node::Engine>>> + Send {
        self(ctx, pool)
    }
}