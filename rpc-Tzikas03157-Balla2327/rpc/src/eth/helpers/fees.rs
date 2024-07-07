//! Contains RPC handler implementations for fee history.

use reth_provider::{BlockIdReader, BlockReaderIdExt, ChainSpecProvider, HeaderProvider};
use reth_rpc_eth_api::helpers::{EthFees, LoadBlock, LoadFee};
use reth_rpc_eth_types::{EthStateCache, FeeHistoryCache, GasPriceOracle};

use crate::EthApi;

/// Implementation of the `EthFees` trait for `EthApi`.
impl<Provider, Pool, Network, EvmConfig> EthFees for EthApi<Provider, Pool, Network, EvmConfig> 
where
    Self: LoadFee,
{
    // This implementation is currently empty, meaning all required trait methods (if any)
    // are implemented by default or in other related trait implementations.
}

/// Implementation of the `LoadFee` trait for `EthApi`.
impl<Provider, Pool, Network, EvmConfig> LoadFee for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: LoadBlock,
    Provider: BlockReaderIdExt + HeaderProvider + ChainSpecProvider,
{
    /// Returns the provider that can fetch block IDs, headers, and chain specifications.
    ///
    /// # Returns
    ///
    /// A provider implementing the `BlockIdReader`, `HeaderProvider`, and `ChainSpecProvider` traits.
    #[inline]
    fn provider(&self) -> impl BlockIdReader + HeaderProvider + ChainSpecProvider {
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

    /// Returns a reference to the gas price oracle.
    ///
    /// # Returns
    ///
    /// A reference to the `GasPriceOracle`.
    #[inline]
    fn gas_oracle(&self) -> &GasPriceOracle<impl BlockReaderIdExt> {
        // Delegates to the inner gas oracle of `EthApi`.
        self.inner.gas_oracle()
    }

    /// Returns a reference to the fee history cache.
    ///
    /// # Returns
    ///
    /// A reference to the `FeeHistoryCache`.
    #[inline]
    fn fee_history_cache(&self) -> &FeeHistoryCache {
        // Delegates to the inner fee history cache of `EthApi`.
        self.inner.fee_history_cache()
    }
}
