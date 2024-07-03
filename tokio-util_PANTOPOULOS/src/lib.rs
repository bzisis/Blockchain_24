//! Event listeners
/// This module provides utilities for setting up and managing event listeners, which are vital 
/// as they allow components to react to blockchain events.
#![doc(

    /// Specifies:
    /// the URL for the logo displayed
    /// the URL for the favicon displayed
    /// Sets the base URL for the issue tracker for this project, directing users to the GitHub issues page for reporting bugs or suggesting features.
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]

/// attribute that warns about unused dependencies, but only when not running tests.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
/// This attribute is used when generating documentation with `docs.rs`.
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Declares the "event_sender" module and the "event_stream" module. 
mod event_sender;
mod event_stream;

/// Making the "EventSender" and the "EventStream" from the corresponding modules, making them publicly accessible.
pub use event_sender::EventSender;
pub use event_stream::EventStream;

/// The Configuration conditional ensures that the `ratelimit` module is included only if the "time" feature is enabled.
#[cfg(feature = "time")]
/// Declares the `ratelimit` module.
pub mod ratelimit;
