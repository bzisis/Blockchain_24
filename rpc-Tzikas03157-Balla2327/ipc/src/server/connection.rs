//! A IPC connection.

use crate::stream_codec::StreamCodec;
use futures::{stream::FuturesUnordered, FutureExt, Sink, Stream};
use std::{
    collections::VecDeque,
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;
use tower::Service;

/// Type alias for a JSON-RPC stream using a framed transport with a custom codec.
pub(crate) type JsonRpcStream<T> = Framed<T, StreamCodec>;

/// Represents an IPC connection with a framed JSON-RPC stream.
#[pin_project::pin_project]
pub(crate) struct IpcConn<T>(#[pin] pub(crate) T);

impl<T> Stream for IpcConn<JsonRpcStream<T>>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = io::Result<String>;

    /// Polls the next item from the stream.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().0.poll_next(cx)
    }
}

impl<T> Sink<String> for IpcConn<JsonRpcStream<T>>
where
    T: AsyncRead + AsyncWrite,
{
    type Error = io::Error;

    /// Polls if the sink is ready to send more data.
    ///
    /// Ensures the underlying framed implementation is flushed to prevent buffering.
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_flush(cx)
    }

    /// Starts sending an item to the sink.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to send.
    fn start_send(self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        self.project().0.start_send(item)
    }

    /// Polls to flush the sink.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_flush(cx)
    }

    /// Polls to close the sink.
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().0.poll_close(cx)
    }
}

/// Drives an [IpcConn] forward.
///
/// This forwards received requests from the connection to the service and sends responses to the
/// connection. This future terminates when the connection is closed.
#[pin_project::pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub(crate) struct IpcConnDriver<T, S, Fut> {
    #[pin]
    pub(crate) conn: IpcConn<JsonRpcStream<T>>,
    pub(crate) service: S,
    /// RPC requests in progress
    #[pin]
    pub(crate) pending_calls: FuturesUnordered<Fut>,
    pub(crate) items: VecDeque<String>,
}

impl<T, S, Fut> IpcConnDriver<T, S, Fut> {
    /// Adds a new item to the send queue.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to add to the queue.
    pub(crate) fn push_back(&mut self, item: String) {
        self.items.push_back(item);
    }
}

impl<T, S> Future for IpcConnDriver<T, S, S::Future>
where
    S: Service<String, Response = Option<String>> + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + Unpin,
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Output = ();

    /// Polls the future, driving the connection forward.
    ///
    /// This handles sending and receiving data, processing requests, and managing the send queue.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        // Items are also pushed from external sources.
        // This will act as a manual yield point to reduce latencies of the polling future that may
        // submit items from an additional source (subscription).
        let mut budget = 5;

        'outer: loop {
            budget -= 1;
            if budget == 0 {
                // Ensure we're woken up again.
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }

            // Write all responses to the sink.
            while this.conn.as_mut().poll_ready(cx).is_ready() {
                if let Some(item) = this.items.pop_front() {
                    if let Err(err) = this.conn.as_mut().start_send(item) {
                        tracing::warn!("IPC response failed: {:?}", err);
                        return Poll::Ready(());
                    }
                } else {
                    break;
                }
            }

            'inner: loop {
                let mut drained = false;
                // Drain all calls that are ready and put them in the output item queue.
                if !this.pending_calls.is_empty() {
                    if let Poll::Ready(Some(res)) = this.pending_calls.as_mut().poll_next(cx) {
                        let item = match res {
                            Ok(Some(resp)) => resp,
                            Ok(None) => continue 'inner,
                            Err(err) => err.into().to_string(),
                        };
                        this.items.push_back(item);
                        continue 'outer;
                    } else {
                        drained = true;
                    }
                }

                // Read from the stream.
                match this.conn.as_mut().poll_next(cx) {
                    Poll::Ready(res) => match res {
                        Some(Ok(item)) => {
                            let mut call = this.service.call(item);
                            match call.poll_unpin(cx) {
                                Poll::Ready(res) => {
                                    let item = match res {
                                        Ok(Some(resp)) => resp,
                                        Ok(None) => continue 'inner,
                                        Err(err) => err.into().to_string(),
                                    };
                                    this.items.push_back(item);
                                    continue 'outer;
                                }
                                Poll::Pending => {
                                    this.pending_calls.push(call);
                                }
                            }
                        }
                        Some(Err(err)) => {
                            tracing::warn!("IPC request failed: {:?}", err);
                            return Poll::Ready(());
                        }
                        None => return Poll::Ready(()),
                    },
                    Poll::Pending => {
                        if drained || this.pending_calls.is_empty() {
                            // At this point all things are pending.
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}
