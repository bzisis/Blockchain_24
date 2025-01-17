//! Implementation of the [`jsonrpsee`] generated [`EthApiServer`](crate::EthApi) trait
//! Handles RPC requests for the `eth_` namespace.

use std::sync::Arc;

use derive_more::Deref;
use reth_primitives::{BlockNumberOrTag, U256};
use reth_provider::BlockReaderIdExt;
use reth_rpc_eth_api::{
    helpers::{transaction::UpdateRawTxForwarder, EthSigner, SpawnBlocking},
    RawTransactionForwarder,
};
use reth_rpc_eth_types::{EthStateCache, FeeHistoryCache, GasCap, GasPriceOracle, PendingBlock};
use reth_tasks::{pool::BlockingTaskPool, TaskSpawner, TokioTaskExecutor};
use tokio::sync::Mutex;

use crate::eth::DevSigner;

/// `Eth` API implementation.
///
/// This type provides the functionality for handling `eth_` related requests.
/// These are implemented two-fold: Core functionality is implemented as
/// [`EthApiSpec`](reth_rpc_eth_api::helpers::EthApiSpec) trait. Additionally, the required server
/// implementations (e.g. [`EthApiServer`](reth_rpc_eth_api::EthApiServer)) are implemented
/// separately in submodules. The rpc handler implementation can then delegate to the main impls.
/// This way [`EthApi`] is not limited to [`jsonrpsee`] and can be used standalone or in other
/// network handlers (for example ipc).
#[derive(Deref)]
pub struct EthApi<Provider, Pool, Network, EvmConfig> {
    /// All nested fields bundled together.
    pub(super) inner: Arc<EthApiInner<Provider, Pool, Network, EvmConfig>>,
}

impl<Provider, Pool, Network, EvmConfig> EthApi<Provider, Pool, Network, EvmConfig>
where
    Provider: BlockReaderIdExt,
{
    /// Creates a new, shareable instance using the default tokio task spawner.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider that can interact with the chain.
    /// * `pool` - The transaction pool.
    /// * `network` - The network interface.
    /// * `eth_cache` - The cache for eth related data.
    /// * `gas_oracle` - The gas oracle for gas price suggestions.
    /// * `gas_cap` - The maximum gas limit for `eth_call` and call tracing RPC methods.
    /// * `eth_proof_window` - The maximum number of blocks into the past for generating state proofs.
    /// * `blocking_task_pool` - The pool dedicated to CPU heavy blocking tasks.
    /// * `fee_history_cache` - The cache for block fees history.
    /// * `evm_config` - The EVM configuration.
    /// * `raw_transaction_forwarder` - Optional transaction forwarder for raw transactions.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: Provider,
        pool: Pool,
        network: Network,
        eth_cache: EthStateCache,
        gas_oracle: GasPriceOracle<Provider>,
        gas_cap: impl Into<GasCap>,
        eth_proof_window: u64,
        blocking_task_pool: BlockingTaskPool,
        fee_history_cache: FeeHistoryCache,
        evm_config: EvmConfig,
        raw_transaction_forwarder: Option<Arc<dyn RawTransactionForwarder>>,
    ) -> Self {
        Self::with_spawner(
            provider,
            pool,
            network,
            eth_cache,
            gas_oracle,
            gas_cap.into().into(),
            eth_proof_window,
            Box::<TokioTaskExecutor>::default(),
            blocking_task_pool,
            fee_history_cache,
            evm_config,
            raw_transaction_forwarder,
        )
    }

    /// Creates a new, shareable instance with a custom task spawner.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider that can interact with the chain.
    /// * `pool` - The transaction pool.
    /// * `network` - The network interface.
    /// * `eth_cache` - The cache for eth related data.
    /// * `gas_oracle` - The gas oracle for gas price suggestions.
    /// * `gas_cap` - The maximum gas limit for `eth_call` and call tracing RPC methods.
    /// * `eth_proof_window` - The maximum number of blocks into the past for generating state proofs.
    /// * `task_spawner` - Custom task spawner for async tasks.
    /// * `blocking_task_pool` - The pool dedicated to CPU heavy blocking tasks.
    /// * `fee_history_cache` - The cache for block fees history.
    /// * `evm_config` - The EVM configuration.
    /// * `raw_transaction_forwarder` - Optional transaction forwarder for raw transactions.
    #[allow(clippy::too_many_arguments)]
    pub fn with_spawner(
        provider: Provider,
        pool: Pool,
        network: Network,
        eth_cache: EthStateCache,
        gas_oracle: GasPriceOracle<Provider>,
        gas_cap: u64,
        eth_proof_window: u64,
        task_spawner: Box<dyn TaskSpawner>,
        blocking_task_pool: BlockingTaskPool,
        fee_history_cache: FeeHistoryCache,
        evm_config: EvmConfig,
        raw_transaction_forwarder: Option<Arc<dyn RawTransactionForwarder>>,
    ) -> Self {
        // Get the block number of the latest block
        let latest_block = provider
            .header_by_number_or_tag(BlockNumberOrTag::Latest)
            .ok()
            .flatten()
            .map(|header| header.number)
            .unwrap_or_default();

        // Create inner struct with provided arguments
        let inner = EthApiInner {
            provider,
            pool,
            network,
            signers: parking_lot::RwLock::new(Default::default()),
            eth_cache,
            gas_oracle,
            gas_cap,
            eth_proof_window,
            starting_block: U256::from(latest_block),
            task_spawner,
            pending_block: Default::default(),
            blocking_task_pool,
            fee_history_cache,
            evm_config,
            raw_transaction_forwarder: parking_lot::RwLock::new(raw_transaction_forwarder),
        };

        // Return new instance of `EthApi`
        Self { inner: Arc::new(inner) }
    }
}

impl<Provider, Pool, Network, EvmConfig> std::fmt::Debug
    for EthApi<Provider, Pool, Network, EvmConfig>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EthApi").finish_non_exhaustive()
    }
}

impl<Provider, Pool, Network, EvmConfig> Clone for EthApi<Provider, Pool, Network, EvmConfig> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<Provider, Pool, Network, EvmConfig> SpawnBlocking
    for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: Clone + Send + Sync + 'static,
{
    /// Returns a handle to the task spawner.
    #[inline]
    fn io_task_spawner(&self) -> impl reth_tasks::TaskSpawner {
        self.inner.task_spawner()
    }

    /// Returns a handle to the blocking task pool.
    #[inline]
    fn tracing_task_pool(&self) -> &reth_tasks::pool::BlockingTaskPool {
        self.inner.blocking_task_pool()
    }
}

impl<Provider, Pool, Network, EvmConfig> EthApi<Provider, Pool, Network, EvmConfig> {
    /// Generates 20 random developer accounts.
    /// Used in DEV mode.
    pub fn with_dev_accounts(&self) {
        let mut signers = self.inner.signers.write();
        *signers = DevSigner::random_signers(20);
    }
}

/// Container type `EthApiInner`
#[allow(missing_debug_implementations)]
pub struct EthApiInner<Provider, Pool, Network, EvmConfig> {
    /// The transaction pool.
    pool: Pool,
    /// The provider that can interact with the chain.
    provider: Provider,
    /// An interface to interact with the network.
    network: Network,
    /// All configured signers.
    signers: parking_lot::RwLock<Vec<Box<dyn EthSigner>>>,
    /// The async cache frontend for eth related data.
    eth_cache: EthStateCache,
    /// The async gas oracle frontend for gas price suggestions.
    gas_oracle: GasPriceOracle<Provider>,
    /// Maximum gas limit for `eth_call` and call tracing RPC methods.
    gas_cap: u64,
    /// The maximum number of blocks into the past for generating state proofs.
    eth_proof_window: u64,
    /// The block number at which the node started.
    starting_block: U256,
    /// The type that can spawn tasks which would otherwise block.
    task_spawner: Box<dyn TaskSpawner>,
    /// Cached pending block if any.
    pending_block: Mutex<Option<PendingBlock>>,
    /// A pool dedicated to CPU heavy blocking tasks.
    blocking_task_pool: BlockingTaskPool,
    /// Cache for block fees history.
    fee_history_cache: FeeHistoryCache,
    /// The type that defines how to configure the EVM.
    evm_config: EvmConfig,
    /// Allows forwarding received raw transactions.
    raw_transaction_forwarder: parking_lot::RwLock<Option<Arc<dyn RawTransactionForwarder>>>,
}

impl<Provider, Pool, Network, EvmConfig> EthApiInner<Provider, Pool, Network, EvmConfig> {
    /// Returns a handle to data on disk.
    #[inline]
    pub const fn provider(&self) -> &Provider {
        &self.provider
    }

    /// Returns a handle to data in memory.
    #[inline]
    pub const fn cache(&self) -> &EthStateCache {
        &self.eth_cache
    }

    /// Returns a handle to the pending block.
    #[inline]
    pub const fn pending_block(&self) -> &Mutex<Option<PendingBlock>> {
        &self.pending_block
    }

    /// Returns a handle to the task spawner.
    #[inline]
    pub const fn task_spawner(&self) -> &dyn TaskSpawner {
        &*self.task_spawner
    }

    /// Returns a handle to the blocking thread pool.
    #[inline]
    pub const fn blocking_task_pool(&self) -> &BlockingTaskPool {
        &self.blocking_task_pool
    }

    /// Returns a handle to the EVM config.
    #[inline]
    pub const fn evm_config(&self) -> &EvmConfig {
        &self.evm_config
    }

    /// Returns a handle to the transaction pool.
    #[inline]
    pub const fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Returns a handle to the transaction forwarder.
    #[inline]
    pub fn raw_tx_forwarder(&self) -> Option<Arc<dyn RawTransactionForwarder>> {
        self.raw_transaction_forwarder.read().clone()
    }

    /// Returns the gas cap.
    #[inline]
    pub const fn gas_cap(&self) -> u64 {
        self.gas_cap
    }

    /// Returns a handle to the gas oracle.
    #[inline]
    pub const fn gas_oracle(&self) -> &GasPriceOracle<Provider> {
        &self.gas_oracle
    }

    /// Returns a handle to the fee history cache.
    #[inline]
    pub const fn fee_history_cache(&self) -> &FeeHistoryCache {
        &self.fee_history_cache
    }

    /// Returns a handle to the signers.
    #[inline]
    pub const fn signers(&self) -> &parking_lot::RwLock<Vec<Box<dyn EthSigner>>> {
        &self.signers
    }

    /// Returns the starting block.
    #[inline]
    pub const fn starting_block(&self) -> U256 {
        self.starting_block
    }

    /// Returns the inner `Network`
    #[inline]
    pub const fn network(&self) -> &Network {
        &self.network
    }

    /// The maximum number of blocks into the past for generating state proofs.
    #[inline]
    pub const fn eth_proof_window(&self) -> u64 {
        self.eth_proof_window
    }
}

impl<Provider, Pool, Network, EvmConfig> UpdateRawTxForwarder
    for EthApiInner<Provider, Pool, Network, EvmConfig>
{
    /// Sets the raw transaction forwarder.
    ///
    /// # Arguments
    ///
    /// * `forwarder` - The raw transaction forwarder.
    fn set_eth_raw_transaction_forwarder(&self, forwarder: Arc<dyn RawTransactionForwarder>) {
        self.raw_transaction_forwarder.write().replace(forwarder);
    }
}

#[cfg(test)]
mod tests {
    use jsonrpsee_types::error::INVALID_PARAMS_CODE;
    use reth_chainspec::BaseFeeParams;
    use reth_evm_ethereum::EthEvmConfig;
    use reth_network_api::noop::NoopNetwork;
    use reth_primitives::{
        constants::ETHEREUM_BLOCK_GAS_LIMIT, Block, BlockNumberOrTag, Header, TransactionSigned,
        B256, U64,
    };
    use reth_provider::{
        test_utils::{MockEthProvider, NoopProvider},
        BlockReader, BlockReaderIdExt, ChainSpecProvider, EvmEnvProvider, StateProviderFactory,
    };
    use reth_rpc_eth_api::EthApiServer;
    use reth_rpc_eth_types::{
        EthStateCache, FeeHistoryCache, FeeHistoryCacheConfig, GasPriceOracle,
    };
    use reth_rpc_server_types::constants::DEFAULT_ETH_PROOF_WINDOW;
    use reth_rpc_types::FeeHistory;
    use reth_tasks::pool::BlockingTaskPool;
    use reth_testing_utils::{generators, generators::Rng};
    use reth_transaction_pool::test_utils::{testing_pool, TestPool};

    use crate::EthApi;

    /// Builds a test instance of `EthApi`.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider that can interact with the chain.
    fn build_test_eth_api<
        P: BlockReaderIdExt
            + BlockReader
            + ChainSpecProvider
            + EvmEnvProvider
            + StateProviderFactory
            + Unpin
            + Clone
            + 'static,
    >(
        provider: P,
    ) -> EthApi<P, TestPool, NoopNetwork, EthEvmConfig> {
        let evm_config = EthEvmConfig::default();
        let cache = EthStateCache::spawn(provider.clone(), Default::default(), evm_config);
        let fee_history_cache =
            FeeHistoryCache::new(cache.clone(), FeeHistoryCacheConfig::default());

        EthApi::new(
            provider.clone(),
            testing_pool(),
            NoopNetwork::default(),
            cache.clone(),
            GasPriceOracle::new(provider, Default::default(), cache),
            ETHEREUM_BLOCK_GAS_LIMIT,
            DEFAULT_ETH_PROOF_WINDOW,
            BlockingTaskPool::build().expect("failed to build tracing pool"),
            fee_history_cache,
            evm_config,
            None,
        )
    }

    /// Prepares the `EthApi` with mock data for testing.
    ///
    /// # Arguments
    ///
    /// * `newest_block` - The newest block number.
    /// * `oldest_block` - The oldest block hash.
    /// * `block_count` - The number of blocks to prepare.
    /// * `mock_provider` - The mock provider to use.
    fn prepare_eth_api(
        newest_block: u64,
        mut oldest_block: Option<B256>,
        block_count: u64,
        mock_provider: MockEthProvider,
    ) -> (EthApi<MockEthProvider, TestPool, NoopNetwork, EthEvmConfig>, Vec<u128>, Vec<f64>) {
        let mut rng = generators::rng();

        // Build mock data
        let mut gas_used_ratios = Vec::new();
        let mut base_fees_per_gas = Vec::new();
        let mut last_header = None;
        let mut parent_hash = B256::default();

        for i in (0..block_count).rev() {
            let hash = rng.gen();
            let gas_limit: u64 = rng.gen();
            let gas_used: u64 = rng.gen();
            // Note: Generates a u32 to avoid overflows later
            let base_fee_per_gas: Option<u64> = rng.gen::<bool>().then(|| rng.gen::<u32>() as u64);

            let header = Header {
                number: newest_block - i,
                gas_limit,
                gas_used,
                base_fee_per_gas,
                parent_hash,
                ..Default::default()
            };
            last_header = Some(header.clone());
            parent_hash = hash;

            let mut transactions = vec![];
            for _ in 0..100 {
                let random_fee: u128 = rng.gen();

                if let Some(base_fee_per_gas) = header.base_fee_per_gas {
                    let transaction = TransactionSigned {
                        transaction: reth_primitives::Transaction::Eip1559(
                            reth_primitives::TxEip1559 {
                                max_priority_fee_per_gas: random_fee,
                                max_fee_per_gas: random_fee + base_fee_per_gas as u128,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    };

                    transactions.push(transaction);
                } else {
                    let transaction = TransactionSigned {
                        transaction: reth_primitives::Transaction::Legacy(Default::default()),
                        ..Default::default()
                    };

                    transactions.push(transaction);
                }
            }

            mock_provider.add_block(
                hash,
                Block { header: header.clone(), body: transactions, ..Default::default() },
            );
            mock_provider.add_header(hash, header);

            oldest_block.get_or_insert(hash);
            gas_used_ratios.push(gas_used as f64 / gas_limit as f64);
            base_fees_per_gas.push(base_fee_per_gas.map(|fee| fee as u128).unwrap_or_default());
        }

        // Add final base fee (for the next block outside of the request)
        let last_header = last_header.unwrap();
        base_fees_per_gas.push(BaseFeeParams::ethereum().next_block_base_fee(
            last_header.gas_used as u128,
            last_header.gas_limit as u128,
            last_header.base_fee_per_gas.unwrap_or_default() as u128,
        ));

        let eth_api = build_test_eth_api(mock_provider);

        (eth_api, base_fees_per_gas, gas_used_ratios)
    }

    /// Tests invalid block range in `fee_history`.
    #[tokio::test]
    async fn test_fee_history_empty() {
        let response = <EthApi<_, _, _, _> as EthApiServer>::fee_history(
            &build_test_eth_api(NoopProvider::default()),
            U64::from(1),
            BlockNumberOrTag::Latest,
            None,
        )
        .await;
        assert!(response.is_err());
        let error_object = response.unwrap_err();
        assert_eq!(error_object.code(), INVALID_PARAMS_CODE);
    }

    /// Tests invalid block range in `fee_history` (request is before genesis).
    #[tokio::test]
    async fn test_fee_history_invalid_block_range_before_genesis() {
        let block_count = 10;
        let newest_block = 1337;
        let oldest_block = None;

        let (eth_api, _, _) =
            prepare_eth_api(newest_block, oldest_block, block_count, MockEthProvider::default());

        let response = <EthApi<_, _, _, _> as EthApiServer>::fee_history(
            &eth_api,
            U64::from(newest_block + 1),
            newest_block.into(),
            Some(vec![10.0]),
        )
        .await;

        assert!(response.is_err());
        let error_object = response.unwrap_err();
        assert_eq!(error_object.code(), INVALID_PARAMS_CODE);
    }

    /// Tests invalid block range in `fee_history` (request is in the future).
    #[tokio::test]
    async fn test_fee_history_invalid_block_range_in_future() {
        let block_count = 10;
        let newest_block = 1337;
        let oldest_block = None;

        let (eth_api, _, _) =
            prepare_eth_api(newest_block, oldest_block, block_count, MockEthProvider::default());

        let response = <EthApi<_, _, _, _> as EthApiServer>::fee_history(
            &eth_api,
            U64::from(1),
            (newest_block + 1000).into(),
            Some(vec![10.0]),
        )
        .await;

        assert!(response.is_err());
        let error_object = response.unwrap_err();
        assert_eq!(error_object.code(), INVALID_PARAMS_CODE);
    }

    /// Tests that requesting no block results in a default response.
    #[tokio::test]
    async fn test_fee_history_no_block_requested() {
        let block_count = 10;
        let newest_block = 1337;
        let oldest_block = None;

        let (eth_api, _, _) =
            prepare_eth_api(newest_block, oldest_block, block_count, MockEthProvider::default());

        let response = <EthApi<_, _, _, _> as EthApiServer>::fee_history(
            &eth_api,
            U64::from(0),
            newest_block.into(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(
            response,
            FeeHistory::default(),
            "none: requesting no block should yield a default response"
        );
    }

    /// Tests that requesting a single block returns 1 block (+ base fee for the next block over).
    #[tokio::test]
    async fn test_fee_history_single_block() {
        let block_count = 10;
        let newest_block = 1337;
        let oldest_block = None;

        let (eth_api, base_fees_per_gas, gas_used_ratios) =
            prepare_eth_api(newest_block, oldest_block, block_count, MockEthProvider::default());

        let fee_history =
            eth_api.fee_history(U64::from(1), newest_block.into(), None).await.unwrap();
        assert_eq!(
            fee_history.base_fee_per_gas,
            &base_fees_per_gas[base_fees_per_gas.len() - 2..],
            "one: base fee per gas is incorrect"
        );
        assert_eq!(
            fee_history.base_fee_per_gas.len(),
            2,
            "one: should return base fee of the next block as well"
        );
        assert_eq!(
            &fee_history.gas_used_ratio,
            &gas_used_ratios[gas_used_ratios.len() - 1..],
            "one: gas used ratio is incorrect"
        );
        assert_eq!(fee_history.oldest_block, newest_block, "one: oldest block is incorrect");
        assert!(
            fee_history.reward.is_none(),
            "one: no percentiles were requested, so there should be no rewards result"
        );
    }

    /// Tests that requesting all blocks returns the correct data.
    #[tokio::test]
    async fn test_fee_history_all_blocks() {
        let block_count = 10;
        let newest_block = 1337;
        let oldest_block = None;

        let (eth_api, base_fees_per_gas, gas_used_ratios) =
            prepare_eth_api(newest_block, oldest_block, block_count, MockEthProvider::default());

        let fee_history =
            eth_api.fee_history(U64::from(block_count), newest_block.into(), None).await.unwrap();

        assert_eq!(
            &fee_history.base_fee_per_gas, &base_fees_per_gas,
            "all: base fee per gas is incorrect"
        );
        assert_eq!(
            fee_history.base_fee_per_gas.len() as u64,
            block_count + 1,
            "all: should return base fee of the next block as well"
        );
        assert_eq!(
            &fee_history.gas_used_ratio, &gas_used_ratios,
            "all: gas used ratio is incorrect"
        );
        assert_eq!(
            fee_history.oldest_block,
            newest_block - block_count + 1,
            "all: oldest block is incorrect"
        );
        assert!(
            fee_history.reward.is_none(),
            "all: no percentiles were requested, so there should be no rewards result"
        );
    }
}
