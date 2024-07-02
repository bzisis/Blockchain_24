// Moysis Moysis Volos, Greece 29/06/2024.

//! Collection of metrics utilities.
//!
//! ## Feature Flags
//!
//! - `common`: Common metrics utilities, such as wrappers around tokio senders and receivers. Pulls
//!   in `tokio`.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",   // URL to the documentation logo
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",                     // URL to the favicon for the documentation
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"                           // Base URL for the issue tracker
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]   // Warn about unused crate dependencies when not testing
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]       // Enable additional documentation features when using docs.rs

/// Metrics derive macro.
pub use reth_metrics_derive::Metrics;   // Re-export the Metrics derive macro from reth_metrics_derive

/// Implementation of common metric utilities.
#[cfg(feature = "common")]
pub mod common;   // Module for common metrics utilities, included if the "common" feature is enabled

/// Re-export core metrics crate.
pub use metrics;   // Re-export the metrics crate
