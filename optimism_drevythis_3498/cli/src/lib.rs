//! OP-Reth CLI implementation.
//! This module provides the implementation of the Optimism Reth CLI.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png", // URL for the logo in the documentation
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256", // URL for the favicon in the documentation
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/" // Base URL for the issue tracker
)]
#![cfg_attr(all(not(test), feature = "optimism"), warn(unused_crate_dependencies))] // Warn about unused dependencies if the `optimism` feature is enabled and not in test mode
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))] // Enable `doc_cfg` and `doc_auto_cfg` features for documentation on docs.rs
#![cfg(feature = "optimism")] // The `optimism` feature must be enabled to use this crate

/// Optimism CLI commands.
/// This module contains the CLI commands for Optimism.
pub mod commands;

pub use commands::{import::ImportOpCommand, import_receipts::ImportReceiptsOpCommand}; // Re-export the `ImportOpCommand` and `ImportReceiptsOpCommand` from the `commands` module

