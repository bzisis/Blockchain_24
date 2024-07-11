//! The entire implementation of the namespace is quite large, hence it is divided across several
//! files.
//!
//! This module provides various helper functionalities specific to the `eth` namespace, including
//! operations related to blocks, transactions, fees, state, and more.

/// Module containing helper functions and utilities for signing transactions.
pub mod signer;

/// Module containing helper functions and utilities related to blocks.
mod block;

/// Module containing helper functions and utilities for making calls.
mod call;

/// Module containing helper functions and utilities related to fee history and gas pricing.
mod fees;

/// Module containing helper functions and utilities related to the Optimism layer 2 solution.
/// This module is included only when the `optimism` feature is enabled.
#[cfg(feature = "optimism")]
pub mod optimism;

/// Module containing helper functions and utilities for working with pending blocks.
/// This module is included only when the `optimism` feature is not enabled.
#[cfg(not(feature = "optimism"))]
mod pending_block;

/// Module containing helper functions and utilities for working with transaction receipts.
/// This module is included only when the `optimism` feature is not enabled.
#[cfg(not(feature = "optimism"))]
mod receipt;

/// Module containing helper functions and utilities for working with Ethereum specifications.
mod spec;

/// Module containing helper functions and utilities for working with Ethereum state.
mod state;

/// Module containing helper functions and utilities for tracing Ethereum transactions and blocks.
mod trace;

/// Module containing helper functions and utilities for working with Ethereum transactions.
mod transaction;
