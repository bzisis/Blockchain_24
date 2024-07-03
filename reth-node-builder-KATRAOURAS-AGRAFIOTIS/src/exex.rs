//! Types for launching execution extensions (ExEx).

use futures::{future::BoxFuture, FutureExt};
use reth_exex::ExExContext;
use reth_node_api::FullNodeComponents;
use std::future::Future;

/// A trait for launching an ExEx.
///
/// This trait defines a method to launch an ExEx, which is an execution extension. The ExEx
/// should be able to run independently and emit events on the channels provided in the 
/// [`ExExContext`].
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
trait LaunchExEx<Node: FullNodeComponents>: Send {
    /// Launches the ExEx.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context containing channels and other utilities necessary for
    ///   running the ExEx.
    ///
    /// # Returns
    ///
    /// This method returns a future that resolves to a result. The result contains another future
    /// which, when executed, runs the ExEx.
    fn launch(
        self,
        ctx: ExExContext<Node>,
    ) -> impl Future<Output = eyre::Result<impl Future<Output = eyre::Result<()>> + Send>> + Send;
}

/// A boxed future type alias for an ExEx.
type BoxExEx = BoxFuture<'static, eyre::Result<()>>;

/// A version of [`LaunchExEx`] that returns a boxed future. This makes the trait object-safe.
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
pub(crate) trait BoxedLaunchExEx<Node: FullNodeComponents>: Send {
    /// Launches the ExEx.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context containing channels and other utilities necessary for
    ///   running the ExEx.
    ///
    /// # Returns
    ///
    /// This method returns a boxed future that resolves to a result. The result contains another
    /// boxed future which, when executed, runs the ExEx.
    fn launch(self: Box<Self>, ctx: ExExContext<Node>) -> BoxFuture<'static, eyre::Result<BoxExEx>>;
}

/// Implements [`BoxedLaunchExEx`] for any type that implements [`LaunchExEx`], [`Send`], and
/// `'static`.
///
/// This implementation allows any type that implements `LaunchExEx` to also be used as a
/// `BoxedLaunchExEx`, returning a [`BoxFuture`] that resolves to a [`BoxExEx`].
///
/// # Type Parameters
///
/// - `E`: The type that implements `LaunchExEx`.
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
impl<E, Node> BoxedLaunchExEx<Node> for E
where
    E: LaunchExEx<Node> + Send + 'static,
    Node: FullNodeComponents,
{
    fn launch(
        self: Box<Self>,
        ctx: ExExContext<Node>,
    ) -> BoxFuture<'static, eyre::Result<BoxExEx>> {
        async move {
            let exex = LaunchExEx::launch(*self, ctx).await?;
            Ok(Box::pin(exex) as BoxExEx)
        }
        .boxed()
    }
}

/// Implements `LaunchExEx` for any closure that takes an [`ExExContext`] and returns a future
/// resolving to an ExEx.
///
/// This implementation allows closures to be used as `LaunchExEx` instances, providing a flexible
/// way to define ExEx launch logic.
///
/// # Type Parameters
///
/// - `Node`: A type that implements the [`FullNodeComponents`] trait.
/// - `F`: The type of the closure.
/// - `Fut`: The future returned by the closure.
/// - `E`: The future returned by the `Fut` future.
impl<Node, F, Fut, E> LaunchExEx<Node> for F
where
    Node: FullNodeComponents,
    F: FnOnce(ExExContext<Node>) -> Fut + Send,
    Fut: Future<Output = eyre::Result<E>> + Send,
    E: Future<Output = eyre::Result<()>> + Send,
{
    fn launch(
        self,
        ctx: ExExContext<Node>,
    ) -> impl Future<Output = eyre::Result<impl Future<Output = eyre::Result<()>> + Send>> + Send
    {
        self(ctx)
    }
}