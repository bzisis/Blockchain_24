use futures_util::StreamExt; // Importing StreamExt for stream extensions
use reth::{
    network::{NetworkEvent, NetworkEvents, NetworkHandle, PeersInfo}, // Importing network types from reth
    rpc::types::PeerId, // Importing PeerId type
};
use reth_network_peers::NodeRecord; // Importing NodeRecord type for network peers
use reth_tokio_util::EventStream; // Importing EventStream for handling asynchronous events
use reth_tracing::tracing::info; // Importing tracing for logging

/// Helper struct for network operations
pub struct NetworkTestContext {
    network_events: EventStream<NetworkEvent>, // Stream for network events
    network: NetworkHandle, // Handle to the network for performing operations
}

impl NetworkTestContext {
    /// Creates a new network helper
    pub fn new(network: NetworkHandle) -> Self {
        // Initialize the network events stream by attaching an event listener to the network handle
        let network_events = network.event_listener();
        // Return initialized network events and network handle
        Self { network_events, network }
    }

    /// Adds a peer to the network node via the network handle
    pub async fn add_peer(&mut self, node_record: NodeRecord) {
        // Use the network handle to add a peer by its ID and TCP address
        self.network.peers_handle().add_peer(node_record.id, node_record.tcp_addr());

        // Await the next network event and match it to confirm the peer addition
        match self.network_events.next().await {
            // If the event is PeerAdded, the peer was added successfully
            Some(NetworkEvent::PeerAdded(_)) => (),
            // If any other event occurs, panic with an error message
            ev => panic!("Expected a peer added event, got: {ev:?}"),
        }
    }

    /// Returns the network node record
    pub fn record(&self) -> NodeRecord {
        // Retrieve the local node record from the network handle
        self.network.local_node_record()
    }

    /// Awaits the next event for an established session
    pub async fn next_session_established(&mut self) -> Option<PeerId> {
        // Loop through the network events
        while let Some(ev) = self.network_events.next().await {
            // Match the event type
            match ev {
                // If a SessionEstablished event occurs, log the event and return the peer ID
                NetworkEvent::SessionEstablished { peer_id, .. } => {
                    info!("Session established with peer: {:?}", peer_id);
                    return Some(peer_id)
                }
                // Continue looping for any other event types
                _ => continue,
            }
        }
        // Return None if no SessionEstablished event occurs
        None
    }
}
