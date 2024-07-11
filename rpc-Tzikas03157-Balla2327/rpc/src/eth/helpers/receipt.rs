//! Builds an RPC receipt response with respect to the data layout of the network.

use reth_rpc_eth_api::helpers::LoadReceipt;
use reth_rpc_eth_types::EthStateCache;

use crate::EthApi;

/// Implementation of the `LoadReceipt` trait for `EthApi`.
impl<Provider, Pool, Network, EvmConfig> LoadReceipt for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: Send + Sync,
{
    /// Returns a reference to the Ethereum state cache.
    ///
    /// This cache contains state information relevant to building the RPC receipt response.
    ///
    /// # Returns
    ///
    /// A reference to the `EthStateCache`.
    #[inline]
    fn cache(&self) -> &EthStateCache {
        // Delegates to the inner cache of `EthApi`.
        self.inner.cache()
    }
}
