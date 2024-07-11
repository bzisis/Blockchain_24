//! Support for building a pending block with transactions from local view of mempool.

use reth_evm::ConfigureEvm;
use reth_provider::{BlockReaderIdExt, ChainSpecProvider, EvmEnvProvider, StateProviderFactory};
use reth_rpc_eth_api::helpers::{LoadPendingBlock, SpawnBlocking};
use reth_rpc_eth_types::PendingBlock;
use reth_transaction_pool::TransactionPool;

use crate::EthApi;

/// Implementation of the `LoadPendingBlock` trait for `EthApi`.
impl<Provider, Pool, Network, EvmConfig> LoadPendingBlock
    for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: SpawnBlocking,
    Provider: BlockReaderIdExt + EvmEnvProvider + ChainSpecProvider + StateProviderFactory,
    Pool: TransactionPool,
    EvmConfig: reth_evm::ConfigureEvm,
{
    /// Returns the provider that can fetch block IDs, headers, chain specifications, and state.
    ///
    /// # Returns
    ///
    /// A provider implementing the `BlockReaderIdExt`, `EvmEnvProvider`, `ChainSpecProvider`, and `StateProviderFactory` traits.
    #[inline]
    fn provider(
        &self,
    ) -> impl BlockReaderIdExt + EvmEnvProvider + ChainSpecProvider + StateProviderFactory {
        // Delegates to the inner provider of `EthApi`.
        self.inner.provider()
    }

    /// Returns the transaction pool.
    ///
    /// # Returns
    ///
    /// A reference to the transaction pool.
    #[inline]
    fn pool(&self) -> impl TransactionPool {
        // Delegates to the inner pool of `EthApi`.
        self.inner.pool()
    }

    /// Returns a reference to the pending block.
    ///
    /// # Returns
    ///
    /// A reference to a mutex-wrapped optional pending block.
    #[inline]
    fn pending_block(&self) -> &tokio::sync::Mutex<Option<PendingBlock>> {
        // Delegates to the inner pending block of `EthApi`.
        self.inner.pending_block()
    }

    /// Returns the EVM configuration.
    ///
    /// # Returns
    ///
    /// A reference to an object implementing the `ConfigureEvm` trait.
    #[inline]
    fn evm_config(&self) -> &impl ConfigureEvm {
        // Delegates to the inner EVM configuration of `EthApi`.
        self.inner.evm_config()
    }
}
