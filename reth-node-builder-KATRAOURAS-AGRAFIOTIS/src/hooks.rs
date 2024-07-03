use crate::node::FullNode;
use reth_node_api::FullNodeComponents;
use std::fmt;

/// Container for all the configurable hook functions.
///
/// This struct allows for the configuration of hooks that are executed at various stages
/// of the node lifecycle. It provides hooks for component initialization and node startup.
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
pub(crate) struct NodeHooks<Node: FullNodeComponents> {
    /// Hook that is run once the node's components are initialized.
    pub(crate) on_component_initialized: Box<dyn OnComponentInitializedHook<Node>>,
    
    /// Hook that is run once the node has started.
    pub(crate) on_node_started: Box<dyn OnNodeStartedHook<Node>>,
    
    /// Marker for the node type.
    pub(crate) _marker: std::marker::PhantomData<Node>,
}

impl<Node: FullNodeComponents> NodeHooks<Node> {
    /// Creates a new, empty [`NodeHooks`] instance for the given node type.
    ///
    /// # Returns
    ///
    /// A new `NodeHooks` instance with default hooks.
    pub(crate) fn new() -> Self {
        Self {
            on_component_initialized: Box::<()>::default(),
            on_node_started: Box::<()>::default(),
            _marker: Default::default(),
        }
    }

    /// Sets the hook that is run once the node's components are initialized.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook function to be set for component initialization.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub(crate) fn set_on_component_initialized<F>(&mut self, hook: F) -> &mut Self
    where
        F: OnComponentInitializedHook<Node> + 'static,
    {
        self.on_component_initialized = Box::new(hook);
        self
    }

    /// Sets the hook that is run once the node's components are initialized.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook function to be set for component initialization.
    ///
    /// # Returns
    ///
    /// An updated `NodeHooks` instance with the new hook.
    #[allow(unused)]
    pub(crate) fn on_component_initialized<F>(mut self, hook: F) -> Self
    where
        F: OnComponentInitializedHook<Node> + 'static,
    {
        self.set_on_component_initialized(hook);
        self
    }

    /// Sets the hook that is run once the node has started.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook function to be set for node startup.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub(crate) fn set_on_node_started<F>(&mut self, hook: F) -> &mut Self
    where
        F: OnNodeStartedHook<Node> + 'static,
    {
        self.on_node_started = Box::new(hook);
        self
    }

    /// Sets the hook that is run once the node has started.
    ///
    /// # Arguments
    ///
    /// * `hook` - The hook function to be set for node startup.
    ///
    /// # Returns
    ///
    /// An updated `NodeHooks` instance with the new hook.
    #[allow(unused)]
    pub(crate) fn on_node_started<F>(mut self, hook: F) -> Self
    where
        F: OnNodeStartedHook<Node> + 'static,
    {
        self.set_on_node_started(hook);
        self
    }
}

impl<Node: FullNodeComponents> Default for NodeHooks<Node> {
    /// Provides a default implementation for `NodeHooks`.
    ///
    /// # Returns
    ///
    /// A new `NodeHooks` instance with default hooks.
    fn default() -> Self {
        Self::new()
    }
}

impl<Node: FullNodeComponents> fmt::Debug for NodeHooks<Node> {
    /// Formats the `NodeHooks` using the given formatter.
    ///
    /// This method formats the `NodeHooks` for debugging purposes, displaying the hooks
    /// for component initialization and node startup.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter used to format the `NodeHooks`.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating whether the formatting was successful or not.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeHooks")
            .field("on_component_initialized", &"...")
            .field("on_node_started", &"...")
            .finish()
    }
}

/// A helper trait for the event hook that is run once the node's components are initialized.
///
/// # Type Parameters
///
/// - `Node`: The type of the node components.
pub trait OnComponentInitializedHook<Node>: Send {
    /// Consumes the event hook and runs it.
    ///
    /// # Arguments
    ///
    /// * `node` - The node components that have been initialized.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` indicating whether the hook was successful or not.
    ///
    /// If this returns an error, the node launch will be aborted.
    fn on_event(self: Box<Self>, node: Node) -> eyre::Result<()>;
}

impl<Node, F> OnComponentInitializedHook<Node> for F
where
    F: FnOnce(Node) -> eyre::Result<()> + Send,
{
    /// Runs the hook function.
    ///
    /// # Arguments
    ///
    /// * `node` - The node components that have been initialized.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` indicating whether the hook was successful or not.
    fn on_event(self: Box<Self>, node: Node) -> eyre::Result<()> {
        (*self)(node)
    }
}

/// A helper trait that is run once the node has started.
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
pub trait OnNodeStartedHook<Node: FullNodeComponents>: Send {
    /// Consumes the event hook and runs it.
    ///
    /// # Arguments
    ///
    /// * `node` - The full node that has started.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` indicating whether the hook was successful or not.
    ///
    /// If this returns an error, the node launch will be aborted.
    fn on_event(self: Box<Self>, node: FullNode<Node>) -> eyre::Result<()>;
}

impl<Node, F> OnNodeStartedHook<Node> for F
where
    Node: FullNodeComponents,
    F: FnOnce(FullNode<Node>) -> eyre::Result<()> + Send,
{
    /// Runs the hook function.
    ///
    /// # Arguments
    ///
    /// * `node` - The full node that has started.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` indicating whether the hook was successful or not.
    fn on_event(self: Box<Self>, node: FullNode<Node>) -> eyre::Result<()> {
        (*self)(node)
    }
}

impl<Node> OnComponentInitializedHook<Node> for () {
    /// Runs the hook function for a unit type.
    ///
    /// # Arguments
    ///
    /// * `_node` - The node components that have been initialized.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` indicating success.
    fn on_event(self: Box<Self>, _node: Node) -> eyre::Result<()> {
        Ok(())
    }
}

impl<Node: FullNodeComponents> OnNodeStartedHook<Node> for () {
    /// Runs the hook function for a unit type.
    ///
    /// # Arguments
    ///
    /// * `_node` - The full node that has started.
    ///
    /// # Returns
    ///
    /// An `eyre::Result<()>` indicating success.
    fn on_event(self: Box<Self>, _node: FullNode<Node>) -> eyre::Result<()> {
        Ok(())
    }
}