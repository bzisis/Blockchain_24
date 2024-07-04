/// Importing `EventStream` from the local crate/module, which is used to handle streams of events.
use crate::EventStream;

/// Importing `broadcast` and `Sender` from `tokio::sync::broadcast`.
/// In Ethereum, this is used for broadcasting messages or events to multiple listeners,
/// which could be nodes in a decentralized network, or components in a wallet application.
use tokio::sync::broadcast::{self, Sender};

/// Importing the `trace` macro from the `tracing` crate.
/// This is useful for debugging, especially in complex systems like Ethereum, where understanding the flow of events can be crucial.
use tracing::trace;

/// Defining a constant for the default size of the broadcast channel buffer,
/// which dictates how many events can be queued for broadcast, (important for handling high volumes of transactions or events in real-time).
const DEFAULT_SIZE_BROADCAST_CHANNEL: usize = 2000;

/// Structure of a bounded broadcast channel for a task, 
/// used to send events (such as blockchain updates or transaction notifications) to multiple receivers.
#[derive(Debug, Clone)]
pub struct EventSender<T> {

    /// The sender part of the broadcast channel, ('T' represents blockchain events).
    sender: Sender<T>,
}

/// Implementing the `Default` trait for `EventSender<T>.
impl<T> Default for EventSender<T>
where

    /// The type `T` must implement `Clone`, `Send`, `Sync`, and be `'static`.
    /// These trait bounds ensure that the events do not contain any non-static references
    /// and can be safely shared across threads,(critical in a multi-threaded environment like a blockchain node or wallet).
    T: Clone + Send + Sync + 'static,
{
    ///Provides a default instance of `EventSender<T>` using the default buffer size.
    fn default() -> Self {

        /// Creates a new EventSender with DEFAULT_SIZE_BROADCAST_CHANNEL
        Self::new(DEFAULT_SIZE_BROADCAST_CHANNEL)
    }
}

///Implementation block for `EventSender<T>` providing methods to work with the broadcast channel.
impl<T: Clone + Send + Sync + 'static> EventSender<T> {

    /// Creates a new `EventSender` (with usize) based on expected event traffic.
    pub fn new(events_channel_size: usize) -> Self {

        /// Creates a broadcast channel with the specified size.
        let (sender, _) = broadcast::channel(events_channel_size);

        ///Returns a new instance of `EventSender` containing the sender part of the channel.
        Self { sender }
    }

    /// Broadcasts an event to all listeners.
    pub fn notify(&self, event: T) {

        /// Attempts to send the event to all receivers/listeners.
        /// If there are no receivers to receive the event, a trace message is logged (helps with debbuging and monitoring).
        if self.sender.send(event).is_err() {
            trace!("no receivers for broadcast events");
        }
    }

    /// Creates a new event stream with a subscriber to the sender as the
    /// receiver.
    pub fn new_listener(&self) -> EventStream<T> {
        EventStream::new(self.sender.subscribe())
    }
}
