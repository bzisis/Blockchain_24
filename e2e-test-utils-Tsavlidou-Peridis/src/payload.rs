use futures_util::StreamExt; // Importing StreamExt for stream extensions
use reth::api::{BuiltPayload, EngineTypes, PayloadBuilderAttributes}; // Importing types from reth API
use reth_payload_builder::{Events, PayloadBuilderHandle, PayloadId}; // Importing types and traits for payload building
use tokio_stream::wrappers::BroadcastStream; // Importing BroadcastStream for handling broadcasted events

/// Helper struct for payload operations
pub struct PayloadTestContext<E: EngineTypes + 'static> {
    pub payload_event_stream: BroadcastStream<Events<E>>, // Stream for payload events
    payload_builder: PayloadBuilderHandle<E>, // Handle to the payload builder for creating and managing payloads
    pub timestamp: u64, // Timestamp for the payload operations
}

impl<E: EngineTypes + 'static> PayloadTestContext<E> {
    /// Creates a new payload helper
    pub async fn new(payload_builder: PayloadBuilderHandle<E>) -> eyre::Result<Self> {
        // Subscribe to payload events and convert them into a stream
        let payload_events = payload_builder.subscribe().await?;
        let payload_event_stream = payload_events.into_stream();
        // Initialize the context with a predefined timestamp
        Ok(Self { payload_event_stream, payload_builder, timestamp: 1710338135 })
    }

    /// Creates a new payload job from static attributes
    pub async fn new_payload(
        &mut self,
        attributes_generator: impl Fn(u64) -> E::PayloadBuilderAttributes,
    ) -> eyre::Result<E::PayloadBuilderAttributes> {
        // Increment the timestamp for each new payload
        self.timestamp += 1;
        // Generate new payload builder attributes using the provided generator function
        let attributes: E::PayloadBuilderAttributes = attributes_generator(self.timestamp);
        // Create a new payload using the payload builder handle
        self.payload_builder.new_payload(attributes.clone()).await.unwrap();
        // Return the generated attributes
        Ok(attributes)
    }

    /// Asserts that the next event is a payload attributes event
    pub async fn expect_attr_event(
        &mut self,
        attrs: E::PayloadBuilderAttributes,
    ) -> eyre::Result<()> {
        // Retrieve the next event from the payload event stream
        let first_event = self.payload_event_stream.next().await.unwrap()?;
        // Check if the event is of type Attributes and assert that the timestamps match
        if let reth::payload::Events::Attributes(attr) = first_event {
            assert_eq!(attrs.timestamp(), attr.timestamp());
        } else {
            panic!("Expect first event as payload attributes.");
        }
        Ok(())
    }

    /// Wait until the best built payload is ready
    pub async fn wait_for_built_payload(&self, payload_id: PayloadId) {
        loop {
            // Retrieve the best built payload using the payload builder handle
            let payload = self.payload_builder.best_payload(payload_id).await.unwrap().unwrap();
            // Check if the payload block's body is empty, if so, wait for a short duration and retry
            if payload.block().body.is_empty() {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                continue
            }
            break
        }
    }

    /// Expects the next event to be a built payload event or panics
    pub async fn expect_built_payload(&mut self) -> eyre::Result<E::BuiltPayload> {
        // Retrieve the next event from the payload event stream
        let second_event = self.payload_event_stream.next().await.unwrap()?;
        // Check if the event is of type BuiltPayload and return it, otherwise panic
        if let reth::payload::Events::BuiltPayload(payload) = second_event {
            Ok(payload)
        } else {
            panic!("Expect a built payload event.");
        }
    }
}
