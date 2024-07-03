//! Standalone crate for Reth configuration and builder types.
//!
//! This crate provides various modules and types for configuring and building
//! different components of the Reth node. It includes support for node event
//! hooks, higher-level node types, node components, and execution extensions (ExExs).
//! Additionally, it re-exports several core configuration traits and API types
//! for convenience.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Node event hooks.
///
/// This module provides types and traits for defining and handling
/// hooks that are executed at various stages of the node lifecycle.
pub mod hooks;

/// Support for configuring the higher level node types.
///
/// This module contains types and functions for configuring
/// and managing higher-level node types within the Reth framework.
pub mod node;
pub use node::*;

/// Support for configuring the components of a node.
///
/// This module provides types and functions for configuring
/// the individual components that make up a Reth node.
pub mod components;

mod builder;
pub use builder::*;

mod launch;
pub use launch::*;

mod handle;
pub use handle::NodeHandle;

/// RPC module.
///
/// This module provides support for configuring and managing
/// the RPC interfaces for the Reth node.
pub mod rpc;

/// Setup module.
///
/// This module contains types and functions for setting up
/// the Reth node environment and initializing its components.
pub mod setup;

/// Support for installing the ExExs (execution extensions) in a node.
///
/// This module provides functionality for installing and managing
/// execution extensions within a Reth node.
pub mod exex;

/// Re-export the core configuration traits.
///
/// This section re-exports several core configuration traits from the
/// `reth_node_core` crate, making them available for use within this crate.
pub use reth_node_core::cli::config::{
    PayloadBuilderConfig, RethNetworkConfig, RethRpcConfig, RethTransactionPoolConfig,
};

// Re-export the core node configuration for convenience.
pub use reth_node_core::node_config::NodeConfig;

// Re-export API types for convenience.
pub use reth_node_api::*;

use aquamarine as _;

use reth_rpc as _;