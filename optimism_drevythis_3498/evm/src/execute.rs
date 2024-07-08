//! Optimism block executor.
//!
//! This module provides functionality for executing Ethereum blocks on the Optimism chain,
//! including transaction execution, receipt generation, and batch execution capabilities.

use crate::{
    l1::ensure_create2_deployer, OptimismBlockExecutionError, OptimismEvmConfig
};
use reth_chainspec::{
    ChainSpec, EthereumHardforks, OptimismHardfork
};
use reth_evm::{
    execute::{
        BatchExecutor, BlockExecutionError, BlockExecutionInput, BlockExecutionOutput,
        BlockExecutorProvider, BlockValidationError, Executor, ProviderError
    },
    system_calls::apply_beacon_root_contract_call,
    ConfigureEvm
};
use reth_execution_types::ExecutionOutcome;
use reth_optimism_consensus::validate_block_post_execution;
use reth_primitives::{
    BlockNumber, BlockWithSenders, Header, Receipt, Receipts, TxType, U256
};
use reth_prune_types::PruneModes;
use reth_revm::{
    batch::{BlockBatchRecord, BlockExecutorStats},
    db::states::bundle_state::BundleRetention,
    state_change::post_block_balance_increments,
    Evm, State
};
use revm_primitives::{
    db::{Database, DatabaseCommit},
    BlockEnv, CfgEnvWithHandlerCfg, EVMError, EnvWithHandlerCfg, ResultAndState
};
use std::sync::Arc;
use tracing::trace;

/// Provides executors to execute regular ethereum blocks
#[derive(Debug, Clone)]
pub struct OpExecutorProvider<EvmConfig = OptimismEvmConfig> {
    chain_spec: Arc<ChainSpec>, // The chain specification
    evm_config: EvmConfig, // Configuration for the EVM
}

impl OpExecutorProvider {
    /// Creates a new default optimism executor provider.
    pub fn optimism(chain_spec: Arc<ChainSpec>) -> Self {
        Self::new(chain_spec, Default::default())
    }
}

impl<EvmConfig> OpExecutorProvider<EvmConfig> {
    /// Creates a new executor provider.
    pub const fn new(chain_spec: Arc<ChainSpec>, evm_config: EvmConfig) -> Self {
        Self { chain_spec, evm_config }
    }
}

impl<EvmConfig> OpExecutorProvider<EvmConfig>
where
    EvmConfig: ConfigureEvm,
{
    /// Constructs an Optimism block executor with the provided database.
    fn op_executor<DB>(&self, db: DB) -> OpBlockExecutor<EvmConfig, DB>
    where
        DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
    {
        OpBlockExecutor::new(
            self.chain_spec.clone(),
            self.evm_config.clone(),
            State::builder()
                .with_database(db)
                .with_bundle_update()
                .without_state_clear()
                .build(),
        )
    }
}

impl<EvmConfig> BlockExecutorProvider for OpExecutorProvider<EvmConfig>
where
    EvmConfig: ConfigureEvm,
{
    type Executor<DB: Database<Error: Into<ProviderError> + std::fmt::Display>> =
        OpBlockExecutor<EvmConfig, DB>;

    type BatchExecutor<DB: Database<Error: Into<ProviderError> + std::fmt::Display>> =
        OpBatchExecutor<EvmConfig, DB>;

    /// Returns a new instance of `OpBlockExecutor` with the provided database.
    fn executor<DB>(&self, db: DB) -> Self::Executor<DB>
    where
        DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
    {
        self.op_executor(db)
    }

    /// Returns a new instance of `OpBatchExecutor` with the provided database.
    fn batch_executor<DB>(&self, db: DB) -> Self::BatchExecutor<DB>
    where
        DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
    {
        let executor = self.op_executor(db);
        OpBatchExecutor {
            executor,
            batch_record: BlockBatchRecord::default(),
            stats: BlockExecutorStats::default(),
        }
    }
}

/// Helper container type for EVM with chain spec.
#[derive(Debug, Clone)]
struct OpEvmExecutor<EvmConfig> {
    chain_spec: Arc<ChainSpec>, // The chain specification
    evm_config: EvmConfig, // Configuration for the EVM
}

impl<EvmConfig> OpEvmExecutor<EvmConfig>
where
    EvmConfig: ConfigureEvm,
{
    /// Executes the transactions in the block and returns the receipts.
    ///
    /// This applies the pre-execution changes, and executes the transactions.
    ///
    /// # Note
    ///
    /// It does __not__ apply post-execution changes.
    fn execute_pre_and_transactions<Ext, DB>(
        &self,
        block: &BlockWithSenders,
        mut evm: Evm<'_, Ext, &mut State<DB>>,
    ) -> Result<(Vec<Receipt>, u64), BlockExecutionError>
    where
        DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
    {
        // Apply pre-execution changes including beacon root contract calls.
        apply_beacon_root_contract_call(
            &self.evm_config,
            &self.chain_spec,
            block.timestamp,
            block.number,
            block.parent_beacon_block_root,
            &mut evm,
        )?;

        // Determine if the block is regolith based on timestamp and hardfork.
        let is_regolith = self.chain_spec.fork(OptimismHardfork::Regolith)
            .active_at_timestamp(block.timestamp);

        // Ensure create2 deployer is activated at the canyon transition.
        ensure_create2_deployer(self.chain_spec.clone(), block.timestamp, evm.db_mut())
            .map_err(|_| OptimismBlockExecutionError::ForceCreate2DeployerFail)?;

        let mut cumulative_gas_used = 0;
        let mut receipts = Vec::with_capacity(block.body.len());

        for (sender, transaction) in block.transactions_with_sender() {
            // Validate transaction gas limit against block available gas.
            let block_available_gas = block.header.gas_limit - cumulative_gas_used;
            if transaction.gas_limit() > block_available_gas &&
                (is_regolith || !transaction.is_system_transaction())
            {
                return Err(BlockValidationError::TransactionGasLimitMoreThanAvailableBlockGas {
                    transaction_gas_limit: transaction.gas_limit(),
                    block_available_gas,
                }.into());
            }

            // Reject blob transactions in an Optimism block.
            if matches!(transaction.tx_type(), TxType::Eip4844) {
                return Err(OptimismBlockExecutionError::BlobTransactionRejected.into());
            }

            // Cache depositor account for deposit nonce handling after regolith hardfork.
            let depositor = (is_regolith && transaction.is_deposit())
                .then(|| {
                    evm.db_mut()
                        .load_cache_account(*sender)
                        .map(|acc| acc.account_info().unwrap_or_default())
                })
                .transpose()
                .map_err(|_| OptimismBlockExecutionError::AccountLoadFailed(*sender))?;

            self.evm_config.fill_tx_env(evm.tx_mut(), transaction, *sender);

            // Execute transaction.
            let ResultAndState { result, state } = evm.transact()
                .map_err(|err| {
                    let new_err = match err {
                        EVMError::Transaction(e) => EVMError::Transaction(e),
                        EVMError::Header(e) => EVMError::Header(e),
                        EVMError::Database(e) => EVMError::Database(e.into()),
                        EVMError::Custom(e) => EVMError::Custom(e),
                        EVMError::Precompile(e) => EVMError::Precompile(e),
                    };
                    BlockValidationError::EVM {
                        hash: transaction.recalculate_hash(),
                        error: Box::new(new_err),
                    }
                })?;

            // Log transaction execution trace.
            trace!(
                target: "evm",
                ?transaction,
                "Executed transaction"
            );

            // Commit transaction state changes.
            evm.db_mut().commit(state);

            // Append gas usage to cumulative gas.
            cumulative_gas_used += result.gas_used();

            // Create and collect transaction receipt.
            receipts.push(Receipt {
                tx_type: transaction.tx_type(),
                success: result.is_success(),
                cumulative_gas_used,
                logs: result.into_logs(),
                deposit_nonce: depositor.map(|account| account.nonce),
                deposit_receipt_version: (transaction.is_deposit() &&
                    self.chain_spec.is_fork_active_at_timestamp(OptimismHardfork::Canyon,
                                                                 block.timestamp))
                    .then_some(1),
            });
        }

        // Drop EVM instance.
        drop(evm);

        Ok((receipts, cumulative_gas_used))
    }
}

/// A basic Ethereum block executor.
///
/// Expected usage:
/// - Create a new instance of the executor.
/// - Execute the block.
#[derive(Debug)]
pub struct OpBlockExecutor<EvmConfig, DB> {
    /// Chain specific evm config that's used to execute a block.
    executor: OpEvmExecutor<EvmConfig>,
    /// The state to use for execution
    state: State<DB>,
}

impl<EvmConfig, DB> OpBlockExecutor<EvmConfig, DB> {
    /// Creates a new Ethereum block executor.
    pub const fn new(chain_spec: Arc<ChainSpec>, evm_config: EvmConfig, state: State<DB>) -> Self {
        Self { executor: OpEvmExecutor { chain_spec, evm_config }, state }
    }

    /// Retrieves the chain specification used by the executor.
    #[inline]
    fn chain_spec(&self) -> &ChainSpec {
        &self.executor.chain_spec
    }

    /// Returns mutable reference to the state that wraps the underlying database.
    #[allow(unused)]
    fn state_mut(&mut self) -> &mut State<DB> {
        &mut self.state
    }
}

impl<EvmConfig, DB> OpBlockExecutor<EvmConfig, DB>
where
    EvmConfig: ConfigureEvm,
    DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
{
    /// Configures a new evm configuration and block environment for the given block.
    ///
    /// Caution: this does not initialize the tx environment.
    fn evm_env_for_block(&self, header: &Header, total_difficulty: U256) -> EnvWithHandlerCfg {
        let mut cfg = CfgEnvWithHandlerCfg::new(Default::default(), Default::default());
        let mut block_env = BlockEnv::default();

        // Fill the configuration and block environment based on the header and difficulty.
        self.executor.evm_config.fill_cfg_and_block_env(
            &mut cfg,
            &mut block_env,
            self.chain_spec(),
            header,
            total_difficulty,
        );

        EnvWithHandlerCfg::new_with_cfg_env(cfg, block_env, Default::default())
    }

    /// Execute a single block and apply the state changes to the internal state.
    ///
    /// Returns the receipts of the transactions in the block and the total gas used.
    ///
    /// Returns an error if execution fails.
    fn execute_without_verification(
        &mut self,
        block: &BlockWithSenders,
        total_difficulty: U256,
    ) -> Result<(Vec<Receipt>, u64), BlockExecutionError> {
        // 1. prepare state on new block
        self.on_new_block(&block.header);

        // 2. configure the evm and execute
        let env = self.evm_env_for_block(&block.header, total_difficulty);

        // Execute transactions within the EVM and retrieve receipts and gas used.
        let (receipts, gas_used) = {
            let evm = self.executor.evm_config.evm_with_env(&mut self.state, env);
            self.executor.execute_pre_and_transactions(block, evm)
        }?;

        // 3. apply post execution changes
        self.post_execution(block, total_difficulty)?;

        Ok((receipts, gas_used))
    }

    /// Apply settings before a new block is executed.
    pub(crate) fn on_new_block(&mut self, header: &Header) {
        // Set state clear flag if the block is after the Spurious Dragon hardfork.
        let state_clear_flag = self.chain_spec().is_spurious_dragon_active_at_block(header.number);
        self.state.set_state_clear_flag(state_clear_flag);
    }

    /// Apply post execution state changes, including block rewards, withdrawals, and irregular DAO
    /// hardfork state change.
    pub fn post_execution(
        &mut self,
        block: &BlockWithSenders,
        total_difficulty: U256,
    ) -> Result<(), BlockExecutionError> {
        // Calculate balance increments based on block information and difficulty.
        let balance_increments =
            post_block_balance_increments(self.chain_spec(), block, total_difficulty);

        // Increment balances in the state.
        self.state
            .increment_balances(balance_increments)
            .map_err(|_| BlockValidationError::IncrementBalanceFailed)?;

        Ok(())
    }
}

impl<EvmConfig, DB> Executor<DB> for OpBlockExecutor<EvmConfig, DB>
where
    EvmConfig: ConfigureEvm,
    DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
{
    type Input<'a> = BlockExecutionInput<'a, BlockWithSenders>;
    type Output = BlockExecutionOutput<Receipt>;
    type Error = BlockExecutionError;
    // Additional implementation for the Executor trait can be added here if needed.
}

/// Executes the block and commits the state changes.
///
/// Returns the receipts of the transactions in the block.
///
/// Returns an error if the block could not be executed or failed verification.
///
/// State changes are committed to the database.
fn execute(mut self, input: Self::Input<'_>) -> Result<Self::Output, Self::Error> {
    // Extract the block and total difficulty from the input
    let BlockExecutionInput { block, total_difficulty } = input;
    
    // Execute the block without verification, capturing receipts and gas used
    let (receipts, gas_used) = self.execute_without_verification(block, total_difficulty)?;

    // Merge state transitions, preserving reverts for bundle retention
    self.state.merge_transitions(BundleRetention::Reverts);

    // Return the block execution output including state, receipts, requests, and gas used
    Ok(BlockExecutionOutput {
        state: self.state.take_bundle(),
        receipts,
        requests: vec![],
        gas_used,
    })
}

/// An executor for a batch of blocks.
///
/// State changes are tracked until the executor is finalized.
#[derive(Debug)]
pub struct OpBatchExecutor<EvmConfig, DB> {
    /// The executor used to execute blocks.
    executor: OpBlockExecutor<EvmConfig, DB>,
    /// Keeps track of the batch and record receipts based on the configured prune mode
    batch_record: BlockBatchRecord,
    /// Statistics tracker for the block executor
    stats: BlockExecutorStats,
}

impl<EvmConfig, DB> OpBatchExecutor<EvmConfig, DB> {
    /// Returns the receipts of the executed blocks.
    pub const fn receipts(&self) -> &Receipts {
        self.batch_record.receipts()
    }

    /// Returns mutable reference to the state that wraps the underlying database.
    #[allow(unused)]
    fn state_mut(&mut self) -> &mut State<DB> {
        self.executor.state_mut()
    }
}

impl<EvmConfig, DB> BatchExecutor<DB> for OpBatchExecutor<EvmConfig, DB>
where
    EvmConfig: ConfigureEvm,
    DB: Database<Error: Into<ProviderError> + std::fmt::Display>,
{
    type Input<'a> = BlockExecutionInput<'a, BlockWithSenders>;
    type Output = ExecutionOutcome;
    type Error = BlockExecutionError;

    /// Executes and verifies one block in the batch.
    ///
    /// This method processes the block, validates post-execution, merges state transitions,
    /// saves receipts, and updates batch metadata.
    ///
    /// Returns an error if execution or validation fails.
    fn execute_and_verify_one(&mut self, input: Self::Input<'_>) -> Result<(), Self::Error> {
        // Extract block and total difficulty from input
        let BlockExecutionInput { block, total_difficulty } = input;
        
        // Execute the block without verification, capturing receipts
        let (receipts, _gas_used) = self.executor.execute_without_verification(block, total_difficulty)?;

        // Validate block post-execution based on chain spec and receipts
        validate_block_post_execution(block, self.executor.chain_spec(), &receipts)?;

        // Determine bundle retention mode for state transitions
        let retention = self.batch_record.bundle_retention(block.number);
        
        // Merge state transitions based on retention mode
        self.executor.state.merge_transitions(retention);

        // Store receipts in the batch record
        self.batch_record.save_receipts(receipts)?;

        // Set the first block number if it's not already set
        if self.batch_record.first_block().is_none() {
            self.batch_record.set_first_block(block.number);
        }

        Ok(())
    }

    /// Finalizes the batch executor and returns the execution outcome.
    ///
    /// This method logs statistics and constructs the final execution outcome
    /// including state, receipts, first block number, and requests.
    fn finalize(mut self) -> Self::Output {
        // Log debug statistics
        self.stats.log_debug();

        // Return execution outcome with final state, receipts, first block, and requests
        ExecutionOutcome::new(
            self.executor.state.take_bundle(),
            self.batch_record.take_receipts(),
            self.batch_record.first_block().unwrap_or_default(),
            self.batch_record.take_requests(),
        )
    }

    /// Sets the tip (latest block number) in the batch record.
    fn set_tip(&mut self, tip: BlockNumber) {
        self.batch_record.set_tip(tip);
    }

    /// Sets the prune modes in the batch record.
    fn set_prune_modes(&mut self, prune_modes: PruneModes) {
        self.batch_record.set_prune_modes(prune_modes);
    }

    /// Provides a size hint for the current state bundle size.
    ///
    /// This method assists in estimating the resource requirements of the executor.
    fn size_hint(&self) -> Option<usize> {
        Some(self.executor.state.bundle_state.size_hint())
    }
}

/// Tests for the OpBatchExecutor and related functionality.
#[cfg(test)]
mod tests {
    use super::*;
    use reth_chainspec::ChainSpecBuilder;
    use reth_primitives::{
        b256, Account, Address, Block, Signature, StorageKey, StorageValue, Transaction,
        TransactionSigned, TxEip1559, BASE_MAINNET,
    };
    use reth_revm::{
        database::StateProviderDatabase, test_utils::StateProviderTest, L1_BLOCK_CONTRACT,
    };
    use std::{collections::HashMap, str::FromStr};

    /// Creates a state provider for testing purposes.
    fn create_op_state_provider() -> StateProviderTest {
        let mut db = StateProviderTest::default();

        // Inserting a mock account for the L1 block contract
        let l1_block_contract_account = Account {
            balance: U256::ZERO,
            bytecode_hash: None,
            nonce: 1,
        };

        let mut l1_block_storage = HashMap::new();
        // Populate mock storage values for the L1 block contract
        l1_block_storage.insert(StorageKey::with_last_byte(1), StorageValue::from(1000000000));
        l1_block_storage.insert(StorageKey::with_last_byte(5), StorageValue::from(188));
        l1_block_storage.insert(StorageKey::with_last_byte(6), StorageValue::from(684000));
        l1_block_storage.insert(
            StorageKey::with_last_byte(3),
            StorageValue::from_str(
                "0x0000000000000000000000000000000000001db0000d27300000000000000005",
            )
            .unwrap(),
        );

        // Insert the account and storage into the database
        db.insert_account(L1_BLOCK_CONTRACT, l1_block_contract_account, None, l1_block_storage);

        db
    }

    /// Provides an OpExecutorProvider for testing purposes.
    fn executor_provider(chain_spec: Arc<ChainSpec>) -> OpExecutorProvider<OptimismEvmConfig> {
        OpExecutorProvider {
            chain_spec,
            evm_config: Default::default(),
        }
    }

    #[test]
    fn op_deposit_fields_pre_canyon() {
        // Define header for testing with mock values
        let header = Header {
            timestamp: 1,
            number: 1,
            gas_limit: 1_000_000,
            gas_used: 42_000,
            receipts_root: b256!(
                "83465d1e7d01578c0d609be33570f91242f013e9e295b0879905346abbd63731"
            ),
            ..Default::default()
        };

        // Create a state provider for testing
        let mut db = create_op_state_provider();

        // Mock account insertion for testing
        let addr = Address::ZERO;
        let account = Account {
            balance: U256::MAX,
            ..Account::default()
        };
        db.insert_account(addr, account, None, HashMap::new());

        // Create a chain spec for testing purposes
        let chain_spec = Arc::new(
            ChainSpecBuilder::from(&*BASE_MAINNET)
                .regolith_activated()
                .build(),
        );

        // Create mock transactions for testing
        let tx = TransactionSigned::from_transaction_and_signature(
            Transaction::Eip1559(TxEip1559 {
                chain_id: chain_spec.chain.id(),
                nonce: 0,
                gas_limit: 21_000,
                to: addr.into(),
                ..Default::default()
            }),
            Signature::default(),
        );

        let tx_deposit = TransactionSigned::from_transaction_and_signature(
            Transaction::Deposit(reth_primitives::TxDeposit {
                from: addr,
                to: addr.into(),
                gas_limit: 21_000,
                ..Default::default()
            }),
            Signature::default(),
        );

        // Create an executor provider for testing
        let provider = executor_provider(chain_spec);
        let mut executor = provider.batch_executor(StateProviderDatabase::new(&db));

        // Load the cache account for the L1 block contract
        executor.state_mut().load_cache_account(L1_BLOCK_CONTRACT).unwrap();

        // Execute and verify a block with mock transactions
        executor
            .execute_and_verify_one((
                &BlockWithSenders {
                    block: Block {
                        header,
                        body: vec![tx, tx_deposit],
                        ommers: vec![],
                        withdrawals: None,
                        requests: None,
                    },
                    senders: vec![addr, addr],
                },
                U256::ZERO,
            )
            .into())
            .unwrap();

        // Access and validate receipts from the executor
        let tx_receipt = executor.receipts()[0][0].as_ref().unwrap();
        let deposit_receipt = executor.receipts()[0][1].as_ref().unwrap();

        // Assert conditions based on receipt contents for pre-canyon transactions
        assert!(deposit_receipt.deposit_receipt_version.is_none());
        assert!(tx_receipt.deposit_receipt_version.is_none());

        assert!(deposit_receipt.deposit_nonce.is_some());
        assert!(tx_receipt.deposit_nonce.is_none());
    }

    #[test]
    fn op_deposit_fields_post_canyon() {
        // Define header for testing with mock values
        let header = Header {
            timestamp: 2,
            number: 1,
            gas_limit: 1_000_000,
            gas_used: 42_000,
            receipts_root: b256!(
                "fffc85c4004fd03c7bfbe5491fae98a7473126c099ac11e8286fd0013f15f908"
            ),
            ..Default::default()
        };

        // Create a state provider for testing
        let mut db = create_op_state_provider();
        let addr = Address::ZERO;
        let account = Account {
            balance: U256::MAX,
            ..Account::default()
        };

        db.insert_account(addr, account, None, HashMap::new());

        // Create a chain spec for testing purposes
        let chain_spec = Arc::new(
            ChainSpecBuilder::from(&*BASE_MAINNET)
                .canyon_activated()
                .build(),
        );

        // Create mock transactions for testing
        let tx = TransactionSigned::from_transaction_and_signature(
            Transaction::Eip1559(TxEip1559 {
                chain_id: chain_spec.chain.id(),
                nonce: 0,
                gas_limit: 21_000,
                to: addr.into(),
                ..Default::default()
            }),
            Signature::default(),
        );

        let tx_deposit = TransactionSigned::from_transaction_and_signature(
            Transaction::Deposit(reth_primitives::TxDeposit {
                from: addr,
                to: addr.into(),
                gas_limit: 21_000,
                ..Default::default()
            }),
            Signature::optimism_deposit_tx_signature(),
        );

        // Create an executor provider for testing
        let provider = executor_provider(chain_spec);
        let mut executor = provider.batch_executor(StateProviderDatabase::new(&db));

        // Load the cache account for the L1 block contract
        executor.state_mut().load_cache_account(L1_BLOCK_CONTRACT).unwrap();

        // Execute and verify a block with mock transactions
        executor
            .execute_and_verify_one((
                &BlockWithSenders {
                    block: Block {
                        header,
                        body: vec![tx, tx_deposit],
                        ommers: vec![],
                        withdrawals: None,
                        requests: None,
                    },
                    senders: vec![addr, addr],
                },
                U256::ZERO,
            )
            .into())
            .expect("Executing a block while canyon is active should not fail");

        // Access and validate receipts from the executor
        let tx_receipt = executor.receipts()[0][0].as_ref().unwrap();
        let deposit_receipt = executor.receipts()[0][1].as_ref().unwrap();

        // Assert conditions based on receipt contents for post-canyon deposit transactions
        assert_eq!(deposit_receipt.deposit_receipt_version, Some(1));
        assert!(tx_receipt.deposit_receipt_version.is_none());

        assert!(deposit_receipt.deposit_nonce.is_some());
        assert!(tx_receipt.deposit_nonce.is_none());
    }
}
