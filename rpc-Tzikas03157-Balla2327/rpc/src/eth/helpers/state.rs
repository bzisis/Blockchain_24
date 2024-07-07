//! Contains RPC handler implementations specific to state.

use reth_provider::StateProviderFactory;
use reth_transaction_pool::TransactionPool;

use reth_rpc_eth_api::helpers::{EthState, LoadState, SpawnBlocking};
use reth_rpc_eth_types::EthStateCache;

use crate::EthApi;

/// Implementation of the `EthState` trait for the `EthApi` struct.
/// This trait provides methods related to the state of the Ethereum network.
impl<Provider, Pool, Network, EvmConfig> EthState for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: LoadState + SpawnBlocking,
{
    /// Returns the maximum proof window for the Ethereum state.
    ///
    /// This method retrieves the proof window from the `eth_proof_window` method.
    ///
    /// # Returns
    ///
    /// A `u64` representing the maximum proof window.
    fn max_proof_window(&self) -> u64 {
        self.eth_proof_window()
    }
}

/// Implementation of the `LoadState` trait for the `EthApi` struct.
/// This trait provides methods to load the state of the Ethereum network.
impl<Provider, Pool, Network, EvmConfig> LoadState for EthApi<Provider, Pool, Network, EvmConfig>
where
    Provider: StateProviderFactory,
    Pool: TransactionPool,
{
    /// Returns the provider for the state.
    ///
    /// This method retrieves the state provider from the inner provider.
    ///
    /// # Returns
    ///
    /// An implementation of `StateProviderFactory`.
    #[inline]
    fn provider(&self) -> impl StateProviderFactory {
        self.inner.provider()
    }

    /// Returns the cache for the Ethereum state.
    ///
    /// This method retrieves the state cache from the inner cache.
    ///
    /// # Returns
    ///
    /// A reference to `EthStateCache`.
    #[inline]
    fn cache(&self) -> &EthStateCache {
        self.inner.cache()
    }

    /// Returns the transaction pool.
    ///
    /// This method retrieves the transaction pool from the inner pool.
    ///
    /// # Returns
    ///
    /// An implementation of `TransactionPool`.
    #[inline]
    fn pool(&self) -> impl TransactionPool {
        self.inner.pool()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use reth_evm_ethereum::EthEvmConfig;
    use reth_primitives::{
        constants::ETHEREUM_BLOCK_GAS_LIMIT, Address, StorageKey, StorageValue, U256,
    };
    use reth_provider::test_utils::{ExtendedAccount, MockEthProvider, NoopProvider};
    use reth_rpc_eth_api::helpers::EthState;
    use reth_rpc_eth_types::{
        EthStateCache, FeeHistoryCache, FeeHistoryCacheConfig, GasPriceOracle,
    };
    use reth_rpc_server_types::constants::DEFAULT_ETH_PROOF_WINDOW;
    use reth_tasks::pool::BlockingTaskPool;
    use reth_transaction_pool::test_utils::testing_pool;

    use super::*;

    /// Tests the storage functionality of the `EthApi` struct.
    ///
    /// This test verifies the behavior of the `storage_at` method for both a `NoopProvider` and a `MockEthProvider`.
    #[tokio::test]
    async fn test_storage() {
        // === Noop ===
        let pool = testing_pool();
        let evm_config = EthEvmConfig::default();

        let cache = EthStateCache::spawn(NoopProvider::default(), Default::default(), evm_config);
        let eth_api = EthApi::new(
            NoopProvider::default(),
            pool.clone(),
            (),
            cache.clone(),
            GasPriceOracle::new(NoopProvider::default(), Default::default(), cache.clone()),
            ETHEREUM_BLOCK_GAS_LIMIT,
            DEFAULT_ETH_PROOF_WINDOW,
            BlockingTaskPool::build().expect("failed to build tracing pool"),
            FeeHistoryCache::new(cache, FeeHistoryCacheConfig::default()),
            evm_config,
            None,
        );

        // Generate a random address
        let address = Address::random();

        // Retrieve storage at the given address with a zero key
        let storage = eth_api.storage_at(address, U256::ZERO.into(), None).await.unwrap();

        // Verify that the storage is zero
        assert_eq!(storage, U256::ZERO.to_be_bytes());

        // === Mock ===
        let mock_provider = MockEthProvider::default();

        // Define a storage value and key
        let storage_value = StorageValue::from(1337);
        let storage_key = StorageKey::random();

        // Create a storage hashmap and add an account to the mock provider
        let storage = HashMap::from([(storage_key, storage_value)]);
        let account = ExtendedAccount::new(0, U256::ZERO).extend_storage(storage);
        mock_provider.add_account(address, account);

        let cache = EthStateCache::spawn(mock_provider.clone(), Default::default(), evm_config);
        let eth_api = EthApi::new(
            mock_provider.clone(),
            pool,
            (),
            cache.clone(),
            GasPriceOracle::new(mock_provider, Default::default(), cache.clone()),
            ETHEREUM_BLOCK_GAS_LIMIT,
            DEFAULT_ETH_PROOF_WINDOW,
            BlockingTaskPool::build().expect("failed to build tracing pool"),
            FeeHistoryCache::new(cache, FeeHistoryCacheConfig::default()),
            evm_config,
            None,
        );

        // Convert the storage key to U256
        let storage_key: U256 = storage_key.into();

        // Retrieve storage at the given address and key
        let storage = eth_api.storage_at(address, storage_key.into(), None).await.unwrap();

        // Verify that the retrieved storage matches the expected value
        assert_eq!(storage, storage_value.to_be_bytes());
    }
}
