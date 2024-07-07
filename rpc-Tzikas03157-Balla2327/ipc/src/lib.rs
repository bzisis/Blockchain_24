//! Reth IPC transport implementation
//!
//! This module provides an IPC transport implementation for Reth, enabling communication over IPC (Inter-Process Communication).
//!
//! ## Feature Flags
//!
//! - `client`: Enables JSON-RPC client support.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Module containing the client-side implementation for IPC transport.
pub mod client;

/// Module containing the server-side implementation for IPC transport.
pub mod server;

/// Module containing the implementation of the JSON codec for streaming protocols.
pub mod stream_codec;
