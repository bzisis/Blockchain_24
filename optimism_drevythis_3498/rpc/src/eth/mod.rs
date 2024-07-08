//! OP-Reth `eth_` endpoint implementation.

use alloy_primitives::{Address, U64}; // Import necessary types from external crates
use reth_chainspec::ChainInfo; // Import necessary types from external crates
use reth_errors::RethResult; // Import necessary types from external crates
use reth_rpc_eth_api::helpers::EthApiSpec; // Import necessary types from external crates
use reth_rpc_types::SyncStatus; // Import necessary types from external crates
use std::future::Future; // Import necessary types from the standard library

/// OP-Reth `Eth` API implementation.
///
/// This type provides functionality for handling `eth_` related requests in OP-Reth.
/// It wraps a default `Eth` implementation, adding features specific to OP-Reth such as
/// transaction forwarding to the sequencer, extended receipts, and additional RPC fields.
#[derive(Debug, Clone)]
pub struct OpEthApi<Eth> {
    inner: Eth, // Inner implementation of the `Eth` API
}

impl<Eth> OpEthApi<Eth> {
    /// Creates a new `OpEthApi` instance from the provided `Eth` implementation.
    pub const fn new(inner: Eth) -> Self {
        Self { inner }
    }
}

impl<Eth: EthApiSpec> EthApiSpec for OpEthApi<Eth> {
    /// Fetches the protocol version asynchronously.
    ///
    /// Delegates to the inner `Eth` implementation.
    fn protocol_version(&self) -> impl Future<Output = RethResult<U64>> + Send {
        self.inner.protocol_version()
    }

    /// Retrieves the chain ID.
    ///
    /// Delegates to the inner `Eth` implementation.
    fn chain_id(&self) -> U64 {
        self.inner.chain_id()
    }

    /// Retrieves information about the chain.
    ///
    /// Delegates to the inner `Eth` implementation.
    fn chain_info(&self) -> RethResult<ChainInfo> {
        self.inner.chain_info()
    }

    /// Retrieves a list of accounts.
    ///
    /// Delegates to the inner `Eth` implementation.
    fn accounts(&self) -> Vec<Address> {
        self.inner.accounts()
    }

    /// Checks if the node is currently syncing.
    ///
    /// Delegates to the inner `Eth` implementation.
    fn is_syncing(&self) -> bool {
        self.inner.is_syncing()
    }

    /// Retrieves the synchronization status.
    ///
    /// Delegates to the inner `Eth` implementation.
    fn sync_status(&self) -> RethResult<SyncStatus> {
        self.inner.sync_status()
    }
}
