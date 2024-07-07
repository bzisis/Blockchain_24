//! Server implementation of the `eth` namespace API.
//! 
//! This module contains the implementation of the Ethereum JSON-RPC API, specifically for the
//! `eth` namespace. It includes various submodules that handle different aspects of the API,
//! such as bundles, core functionality, filters, and pub/sub mechanisms.

/// Submodule for handling Ethereum bundles.
pub mod bundle;

/// Submodule for core Ethereum API functionality.
pub mod core;

/// Submodule for handling Ethereum filters.
pub mod filter;

/// Submodule containing helper functions and utilities.
pub mod helpers;

/// Submodule for handling Ethereum pub/sub mechanisms.
pub mod pubsub;

/// Implementation of `eth` namespace API.
/// 
/// The `EthBundle` struct provides the implementation for handling Ethereum transaction bundles.
/// It encapsulates the logic for simulating and processing bundles of transactions.
pub use bundle::EthBundle;

/// Core Ethereum API implementation.
/// 
/// The `EthApi` struct provides the core functionality for handling various Ethereum JSON-RPC
/// requests. It interacts with the underlying provider, transaction pool, and network to
/// process requests and return appropriate responses.
pub use core::EthApi;

/// Implementation of Ethereum filters.
/// 
/// The `EthFilter` struct handles the creation, management, and querying of Ethereum filters.
/// Filters allow clients to monitor specific blockchain events, such as new blocks or logs
/// matching certain criteria. The `EthFilterConfig` struct provides configuration options for
/// filters.
pub use filter::{EthFilter, EthFilterConfig};

/// Implementation of Ethereum pub/sub mechanisms.
/// 
/// The `EthPubSub` struct provides the implementation for Ethereum pub/sub functionality.
/// It allows clients to subscribe to various events and receive real-time notifications.
pub use pubsub::EthPubSub;

/// Helper for managing developer signers.
/// 
/// The `DevSigner` struct provides utilities for creating and managing developer accounts
/// and signers. It is useful for testing and development purposes.
pub use helpers::signer::DevSigner;

/// Trait for forwarding raw Ethereum transactions.
/// 
/// The `RawTransactionForwarder` trait defines the interface for forwarding raw Ethereum
/// transactions. It allows for custom implementations that can handle the forwarding of
/// transactions to the Ethereum network.
pub use reth_rpc_eth_api::RawTransactionForwarder;
