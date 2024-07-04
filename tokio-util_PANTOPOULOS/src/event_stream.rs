//! Event streams related functionality.
/// This module provides functionality to handle streams of events, crucial for Ethereum applications.


use std::{

    ///  Imports for dealing with pinned memory locations
    ///  `Pin` is used to prevent data from being moved in memory, which is important for safe asynchronous operations.
    pin::Pin,

    /// Imports for working with asynchronous tasks, particularly for polling the state of tasks.
    task::{Context, Poll},
};

/// The `Stream` trait from `tokio_stream` is used for defining asynchronous streams.
use tokio_stream::Stream;
/// The `warn` macro from the `tracing` crate is used for logging warning messages.
use tracing::warn;

/// Thin wrapper around tokio's `BroadcastStream` to allow skipping broadcast errors.
/// Important where stream resilience is critical and not every dropped message needs to interrupt the processing flow
#[derive(Debug)]

/// The inner broadcast stream
pub struct EventStream<T> {
    inner: tokio_stream::wrappers::BroadcastStream<T>,
}

///Implementation block for `EventStream<T>`
impl<T> EventStream<T>
where

    /// Trait bounds for type <T>.
    /// Clonable, able to be send across threads and it must not contain any non-static references.
    T: Clone + Send + 'static,
{
    /// Creates a new `EventStream`, (from a broadcast receiver).
    pub fn new(receiver: tokio::sync::broadcast::Receiver<T>) -> Self {

        /// Wraps the provided receiver in a `BroadcastStream` to handle streaming of events
        /// and returns a new instance of `EventStream` containing the wrapped stream.
        let inner = tokio_stream::wrappers::BroadcastStream::new(receiver);
        Self { inner }
    }
}

/// Implementation of the `Stream` trait for `EventStream<T>`, allowing it to be used as an asynchronous stream.
impl<T> Stream for EventStream<T>
where

    /// The type `T` must implement `Clone`, `Send`, and be `'static`.
    T: Clone + Send + 'static,
{   
    /// Defines the type of streamed items to <T>.
    type Item = T;

    /// Polls the next item from the stream, processing it according to its state (By Pattern Matching).
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // Pinning the mutable reference to `self.inner` to the stack,
            // ensuring that the `BroadcastStream` is safely accessed without moving in memory.
            match Pin::new(&mut self.inner).poll_next(cx) {

                /// Pattern matching on the result of polling the next item from `BroadcastStream`.
                /// If the poll is ready and successfully returns an item (`Ok(item)`), 
                /// the item is immediately returned wrapped in `Poll::Ready(Some(item))`.
                /// This means the stream has produced a valid value to be processed.
                Poll::Ready(Some(Ok(item))) => return Poll::Ready(Some(item)),

                /// If the poll is ready but returns an error (`Err(e)`), it means there was a lag error,
                /// (The channel buffer might be full and some messages might be dropped).
                /// The warning log captures this issue without interrupting the stream processing.
                /// The `continue` statement allows the loop to skip the error and attempt to poll the next item
                Poll::Ready(Some(Err(e))) => {
                    warn!("BroadcastStream lagged: {e:?}");
                    continue
                }

                /// If the stream is complete and has no more items to proccess, it returns "Poll::Ready(None)",
                /// indicating that the stream has ended and no further events will be received.
                Poll::Ready(None) => return Poll::Ready(None),

                /// If the stream is not ready to provide an item (`Poll::Pending`), it returns `Poll::Pending`.
                /// This informs the asynchronous runtime that the task should be polled again when more events might be ready to be processed.
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]

/// Defines a module for testing the `EventStream` functionality and reliability.
mod tests {

    /// Super brings all items from the parent module into scope for testing.
    use super::*;
    /// Imports `broadcast` from `tokio::sync` to create channels for testing.
    use tokio::sync::broadcast;
    /// Imports additional methods for working with streams (like ".collect()").
    use tokio_stream::StreamExt;

    #[tokio::test]

    /// An asynchronous test function to validate that `EventStream` yields items correctly.
    async fn test_event_stream_yields_items() {

        /// Creates a broadcast channel with a buffer size of 16.
        let (tx, _) = broadcast::channel(16);
        /// Creates an `EventStream` from a broadcast receiver.
        let my_stream = EventStream::new(tx.subscribe());

        // Sends three items into the channel. (events like transactions or block data).
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        // drop the sender to terminate the stream and allow collect to work.
        drop(tx);

        /// Collects all items from the stream into a vector.
        let items: Vec<i32> = my_stream.collect().await;

        /// Asserts that the collected items match the expected values
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[tokio::test]
    /// An asynchronous test function to ensure that `EventStream` skips lag errors correctly.
    async fn test_event_stream_skips_lag_errors() {

        /// Creates a broadcast channel with a small buffer size of 2 (simulate lagging conditions).
        let (tx, _) = broadcast::channel(2);
        /// Creates an `EventStream` from a broadcast receiver.
        let my_stream = EventStream::new(tx.subscribe());

        /// Creates additional subscribers(mutli-listener case)
        let mut _rx2 = tx.subscribe();
        let mut _rx3 = tx.subscribe();

        /// Sends multiple items into the channel. The buffer size is exceeded to simulate lagging.
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        tx.send(4).unwrap(); // This will cause lag for the first subscriber

        // drop the sender to terminate the stream and allow collect to work.
        drop(tx);

        // Ensure lag errors are skipped and only valid items are collected
        let items: Vec<i32> = my_stream.collect().await;

        /// Asserts that only the items sent after the buffer was exceeded are collected, indicating lag errors were skipped.
        assert_eq!(items, vec![3, 4]);
    }
}
