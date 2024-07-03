//! Ethereum fork types used in reth.
//!
//! This crate contains Ethereum fork types and helper functions.
//!
//! ## Feature Flags
//!
//! - `arbitrary`: Adds `proptest` and `arbitrary` support for primitive types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
/// Warns about unused crate dependencies except in test mode
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
/// Enables experimental documentation features for cfg and auto-cfg
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
/// Disables standard library if feature "std" is not enabled
#![cfg_attr(not(feature = "std"), no_std)]

/// Import necessary dependencies when std feature is disabled
#[cfg(not(feature = "std"))]
extern crate alloc;

/// Module declarations
mod display;
mod forkcondition;
mod forkid;
mod hardfork;
mod hardforks;
mod head;

/// Public exports from the crate
pub use forkid::{
    EnrForkIdEntry, ForkFilter, ForkFilterKey, ForkHash, ForkId, ForkTransition, ValidationError,
};
/// Exports related to hardforks
pub use hardfork::{EthereumHardfork, Hardfork, OptimismHardfork, DEV_HARDFORKS};
/// Export the Head structure representing Ethereum block headers
pub use head::Head;

pub use display::DisplayHardforks;      /// Export for displaying hardforks
pub use forkcondition::ForkCondition;   /// Export for fork conditions
pub use hardforks::*;                   /// Export all hardforks definitions

/// Public exports when the "arbitrary" feature is enabled (for testing)
#[cfg(any(test, feature = "arbitrary"))]
pub use arbitrary;
