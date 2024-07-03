use crate::node::FullNode;
use reth_node_api::FullNodeComponents;
use reth_node_core::exit::NodeExitFuture;
use std::fmt;

/// A handle to the launched node.
///
/// This struct provides a handle to the node and allows you to wait for the node to exit.
/// It contains all the components of the node and an exit future that can be awaited.
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
#[must_use = "Needs to await the node exit future"]
pub struct NodeHandle<Node: FullNodeComponents> {
    /// All node components.
    ///
    /// This field holds all the components of the node.
    pub node: FullNode<Node>,
    
    /// The exit future of the node.
    ///
    /// This future can be awaited to detect when the node has exited.
    pub node_exit_future: NodeExitFuture,
}

impl<Node: FullNodeComponents> NodeHandle<Node> {
    /// Waits for the node to exit, if it was configured to exit.
    ///
    /// This async function waits for the node's exit future to complete and returns the result.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` that indicates whether waiting for the node to exit was successful or not.
    pub async fn wait_for_node_exit(self) -> eyre::Result<()> {
        self.node_exit_future.await
    }
}

impl<Node: FullNodeComponents> fmt::Debug for NodeHandle<Node> {
    /// Formats the `NodeHandle` using the given formatter.
    ///
    /// This method formats the `NodeHandle` for debugging purposes, displaying the node components and
    /// the node exit future.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter used to format the `NodeHandle`.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating whether the formatting was successful or not.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeHandle")
            .field("node", &"...")
            .field("node_exit_future", &self.node_exit_future)
            .finish()
    }
}