//! Contains RPC handler implementations specific to blocks.

use reth_provider::{BlockReaderIdExt, HeaderProvider};
use reth_rpc_eth_api::helpers::{EthBlocks, LoadBlock, LoadPendingBlock, SpawnBlocking};
use reth_rpc_eth_types::EthStateCache;

use crate::EthApi;

impl<Provider, Pool, Network, EvmConfig> EthBlocks for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: LoadBlock,
    Provider: HeaderProvider,
{
    /// Returns the provider that can fetch block headers.
    ///
    /// # Returns
    ///
    /// A provider implementing the `reth_provider::HeaderProvider` trait.
    #[inline]
    fn provider(&self) -> impl reth_provider::HeaderProvider {
        // Delegates to the inner provider of `EthApi`.
        self.inner.provider()
    }
}

impl<Provider, Pool, Network, EvmConfig> LoadBlock for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: LoadPendingBlock + SpawnBlocking,
    Provider: BlockReaderIdExt,
{
    /// Returns the provider that can fetch block headers and IDs.
    ///
    /// # Returns
    ///
    /// A provider implementing the `BlockReaderIdExt` trait.
    #[inline]
    fn provider(&self) -> impl BlockReaderIdExt {
        // Delegates to the inner provider of `EthApi`.
        self.inner.provider()
    }

    /// Returns a reference to the Ethereum state cache.
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
