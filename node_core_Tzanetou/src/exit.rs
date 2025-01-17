//! Helper types for waiting for the node to exit.
//! 
//! This module defines a `NodeExitFuture` type which is a future that resolves when the node
//! exits. This future can be polled to wait for the consensus engine to exit and it supports 
//! an optional termination flag indicating whether the node should be terminated after the 
//! pipeline sync.

use futures::{future::BoxFuture, FutureExt};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
};

/// A Future which resolves when the node exits
pub struct NodeExitFuture {
    /// The consensus engine future.
    /// This can be polled to wait for the consensus engine to exit.
    consensus_engine_fut: Option<BoxFuture<'static, eyre::Result<()>>>,

    /// Flag indicating whether the node should be terminated after the pipeline sync.
    terminate: bool,
}

impl fmt::Debug for NodeExitFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeExitFuture")
            .field("consensus_engine_fut", &"...")
            .field("terminate", &self.terminate)
            .finish()
    }
}

impl NodeExitFuture {
    /// Create a new `NodeExitFuture`.
    pub fn new<F>(consensus_engine_fut: F, terminate: bool) -> Self
    where
        F: Future<Output = eyre::Result<()>> + 'static + Send,
    {
        Self { consensus_engine_fut: Some(Box::pin(consensus_engine_fut)), terminate }
    }
}

impl Future for NodeExitFuture {
    type Output = eyre::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(rx) = this.consensus_engine_fut.as_mut() {
            // Poll the consensus engine future
            match ready!(rx.poll_unpin(cx)) {
                 // If the future resolves successfully, take it out and check the terminate flag
                Ok(_) => {
                    this.consensus_engine_fut.take();
                    if this.terminate {
                        Poll::Ready(Ok(()))
                    } else {
                        Poll::Pending
                    }
                }
                Err(err) => Poll::Ready(Err(err)),
            }
        } else {
            // If the consensus engine future is already taken, keep the NodeExitFuture pending
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::poll_fn;

    /// Test the case where the terminate flag is true
    #[tokio::test]
    async fn test_node_exit_future_terminate_true() {
        let fut = async { Ok(()) };

        let node_exit_future = NodeExitFuture::new(fut, true);

        // Await the NodeExitFuture and check if it resolves with Ok
        let res = node_exit_future.await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_node_exit_future_terminate_false() {
        let fut = async { Ok(()) };

        let mut node_exit_future = NodeExitFuture::new(fut, false);

        // Poll the NodeExitFuture and check if it remains pending
        poll_fn(|cx| {
            assert!(node_exit_future.poll_unpin(cx).is_pending());
            Poll::Ready(())
        })
        .await;
    }
}
