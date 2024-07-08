//! Standalone crate for Optimism-specific Reth primitive types.
//!
//! This crate provides essential types and functionalities specific to Optimism,
//! tailored for interacting with the Reth protocol.
//!
//! The crate includes:
//! - `bedrock_import`: A module containing specific imports related to Bedrock.
//!
//! The crate is designed to be used as a standalone library for integrating Optimism's
//! features into broader applications or systems.
//!
//! HTML documentation features:
//! - Logo URL: https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png
//! - Favicon URL: https://avatars0.githubusercontent.com/u/97369466?s=256
//! - Issue Tracker Base URL: https://github.com/paradigmxyz/reth/issues/
//!
//! Feature flags used:
//! - `docsrs`: Enables conditional compilation for documentation generation features.
//!   - `doc_cfg`: Allows conditional inclusion of items based on feature flags.
//!   - `doc_auto_cfg`: Automatically enables feature flags based on `cargo` settings.
//!
//! This setup ensures that the crate's documentation is informative and accessible,
//! providing necessary details and links for understanding and contributing to the project.
//!
//! The `bedrock_import` module within this crate contains specific implementations
//! and utilities related to integrating with Bedrock, an essential component of the
//! Optimism protocol.
//!
//! Example usage:
//! ```rust
//! use optimism_reth::bedrock_import::*;
//!
//! // Use types and functions provided by the `bedrock_import` module.
//! ```
//!
//! For more details, refer to the project's [GitHub repository](https://github.com/paradigmxyz/reth).
//!
//! For reporting issues or feature requests, visit the [issue tracker](https://github.com/paradigmxyz/reth/issues).
//!
//! This crate is foundational for developers looking to build applications or tooling
//! around Optimism's blockchain solutions, leveraging its specialized types and functionalities.
//!
//! Developed and maintained by Paradigm XYZ.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod bedrock_import;
