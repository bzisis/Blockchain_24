// Author: Zisis Balatsos

//! transaction-pool integration tests

///
/// This module contains integration tests for the transaction pool. These tests ensure that the
/// transaction pool works correctly in various scenarios including handling blobs, evicting
/// transactions, processing listeners, and managing pending transactions.
///

/// Integration tests for handling blob transactions
#[cfg(feature = "test-utils")]
mod blobs;

/// Integration tests for evicting transactions from the pool
#[cfg(feature = "test-utils")]
mod evict;

/// Integration tests for transaction listeners
#[cfg(feature = "test-utils")]
mod listeners;

/// Integration tests for managing pending transactions
#[cfg(feature = "test-utils")]
mod pending;

/// Main entry point for the test suite.
///
/// This function is required to ensure that the test suite is correctly recognized by the test
/// framework. Since the integration tests are defined in modules, the main function does not
/// perform any operations.
///
const fn main() {}
