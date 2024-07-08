//! Standalone crate for Optimism-specific Reth configuration and builder types.
//!
//! This crate provides configuration and builder types specific to Optimism for the Reth blockchain node.
//!
//! # Features
//!
//! - `optimism`: Enables Optimism-specific features and implementations.
//!
//! # Documentation Attributes
//!
//! - `html_logo_url`: Sets the URL for the HTML documentation logo.
//! - `html_favicon_url`: Sets the URL for the HTML documentation favicon.
//! - `issue_tracker_base_url`: Sets the base URL for linking issues in documentation.
//!
//! # Configuration
//!
//! - This crate requires the `optimism` feature to be enabled to use its functionalities.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// The `optimism` feature must be enabled to use this crate.
#![cfg(feature = "optimism")]

/// CLI argument parsing for the optimism node.
pub mod args;

/// Exports optimism-specific implementations of the [`EngineTypes`](reth_node_api::EngineTypes)
/// trait.
pub mod engine;
pub use engine::OptimismEngineTypes;

/// Contains the OptimismNode struct, which represents the Optimism node configuration and functionality.
pub mod node;
pub use node::OptimismNode;

/// Transaction pool management specific to Optimism.
pub mod txpool;

/// RPC (Remote Procedure Call) implementations for interacting with the Optimism node.
pub mod rpc;

/// Re-exports from `reth_optimism_payload_builder` for building Optimism payloads.
pub use reth_optimism_payload_builder::{
    OptimismBuiltPayload, OptimismPayloadBuilder, OptimismPayloadBuilderAttributes,
};

/// Re-exports from `reth_evm_optimism` for Optimism-specific EVM (Ethereum Virtual Machine) implementations.
pub use reth_evm_optimism::*;
