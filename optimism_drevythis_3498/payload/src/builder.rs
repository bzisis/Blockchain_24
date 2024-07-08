//! Optimism payload builder implementation.

use crate::{
    error::OptimismPayloadBuilderError, // Importing error handling related to Optimism payload building
    payload::{OptimismBuiltPayload, OptimismPayloadBuilderAttributes}, // Importing payload and attributes related to Optimism
};
use reth_basic_payload_builder::*; // Importing basic payload builder utilities
use reth_chainspec::{ChainSpec, EthereumHardforks, OptimismHardfork}; // Importing chain specification related to Ethereum and Optimism
use reth_evm::{system_calls::pre_block_beacon_root_contract_call, ConfigureEvm}; // Importing EVM system calls and configuration utilities
use reth_execution_types::ExecutionOutcome; // Importing execution outcome types
use reth_payload_builder::error::PayloadBuilderError; // Importing payload builder error handling
use reth_primitives::{
    constants::{BEACON_NONCE, EMPTY_RECEIPTS, EMPTY_TRANSACTIONS}, // Importing constants like nonce, empty receipts, and transactions
    eip4844::calculate_excess_blob_gas, // Importing EIP-4844 related utilities
    proofs, Block, Header, IntoRecoveredTransaction, Receipt, TxType, EMPTY_OMMER_ROOT_HASH, U256, // Importing various primitives like Block, Header, Receipt, etc.
};
use reth_provider::StateProviderFactory; // Importing state provider factory
use reth_revm::database::StateProviderDatabase; // Importing state provider database for REVM
use reth_transaction_pool::{BestTransactionsAttributes, TransactionPool}; // Importing transaction pool related attributes
use revm::{
    db::states::bundle_state::BundleRetention, // Importing bundle state related to REVM
    primitives::{EVMError, EnvWithHandlerCfg, InvalidTransaction, ResultAndState}, // Importing REVM primitives like EVMError, InvalidTransaction, etc.
    DatabaseCommit, State, // Importing database commit and State related to REVM
};
use std::sync::Arc; // Importing Arc for atomic reference counting
use tracing::{debug, trace, warn}; // Importing tracing utilities for logging

/// Optimism's payload builder
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimismPayloadBuilder<EvmConfig> {
    /// The rollup's compute pending block configuration option.
    // TODO(clabby): Implement this feature.
    compute_pending_block: bool,
    /// The rollup's chain spec.
    chain_spec: Arc<ChainSpec>,
    /// The type responsible for creating the evm.
    evm_config: EvmConfig,
}

impl<EvmConfig> OptimismPayloadBuilder<EvmConfig> {
    /// `OptimismPayloadBuilder` constructor.
    pub const fn new(chain_spec: Arc<ChainSpec>, evm_config: EvmConfig) -> Self {
        Self { compute_pending_block: true, chain_spec, evm_config }
    }

    /// Sets the rollup's compute pending block configuration option.
    pub const fn set_compute_pending_block(mut self, compute_pending_block: bool) -> Self {
        self.compute_pending_block = compute_pending_block;
        self
    }

    /// Enables the rollup's compute pending block configuration option.
    pub const fn compute_pending_block(self) -> Self {
        self.set_compute_pending_block(true)
    }

    /// Returns the rollup's compute pending block configuration option.
    pub const fn is_compute_pending_block(&self) -> bool {
        self.compute_pending_block
    }

    /// Sets the rollup's chainspec.
    pub fn set_chain_spec(mut self, chain_spec: Arc<ChainSpec>) -> Self {
        self.chain_spec = chain_spec;
        self
    }
}

/// Implementation of the `PayloadBuilder` trait for `OptimismPayloadBuilder`.
impl<Pool, Client, EvmConfig> PayloadBuilder<Pool, Client> for OptimismPayloadBuilder<EvmConfig>
where
    Client: StateProviderFactory,
    Pool: TransactionPool,
    EvmConfig: ConfigureEvm,
{
    type Attributes = OptimismPayloadBuilderAttributes;
    type BuiltPayload = OptimismBuiltPayload;

    /// Attempts to build an Optimism payload using the provided arguments.
    ///
    /// This function calls `optimism_payload_builder` to construct the payload based on the
    /// configuration and pending block computation.
    fn try_build(
        &self,
        args: BuildArguments<Pool, Client, OptimismPayloadBuilderAttributes, OptimismBuiltPayload>,
    ) -> Result<BuildOutcome<OptimismBuiltPayload>, PayloadBuilderError> {
        optimism_payload_builder(self.evm_config.clone(), args, self.compute_pending_block)
    }

    /// Defines behavior when a payload is missing during the build process.
    ///
    /// For this implementation, it waits for the job already in progress (`AwaitInProgress`),
    /// as racing another job doesn't provide any benefit.
    fn on_missing_payload(
        &self,
        _args: BuildArguments<Pool, Client, OptimismPayloadBuilderAttributes, OptimismBuiltPayload>,
    ) -> MissingPayloadBehaviour<Self::BuiltPayload> {
        // we want to await the job that's already in progress because that should be returned as
        // is, there's no benefit in racing another job
        MissingPayloadBehaviour::AwaitInProgress
    }

    /// Builds an empty payload using provided client and configuration.
    ///
    /// This function constructs an empty payload using details from the provided `config`.
    /// It fetches necessary state information and applies specific operations such as
    /// EIP-4788 pre-block contract call and withdrawals handling.
    fn build_empty_payload(
        &self,
        client: &Client,
        config: PayloadConfig<Self::Attributes>,
    ) -> Result<OptimismBuiltPayload, PayloadBuilderError> {
        let extra_data = config.extra_data();
        let PayloadConfig {
            initialized_block_env,
            parent_block,
            attributes,
            chain_spec,
            initialized_cfg,
            ..
        } = config;

        // Debug logging: Logs information about building an empty payload.
        debug!(
            target: "payload_builder",
            parent_hash = ?parent_block.hash(),
            parent_number = parent_block.number,
            "building empty payload"
        );

        // Retrieves state from client using parent block hash.
        let state = client.state_by_block_hash(parent_block.hash()).map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to get state for empty payload"
            );
            err
        })?;

        // Constructs a state database (`db`) from the retrieved state.
        let mut db = State::builder()
            .with_database(StateProviderDatabase::new(state))
            .with_bundle_update()
            .build();

        // Converts `initialized_block_env` fields into appropriate types.
        let base_fee = initialized_block_env.basefee.to::<u64>();
        let block_number = initialized_block_env.number.to::<u64>();
        let block_gas_limit: u64 = initialized_block_env.gas_limit.try_into().unwrap_or(u64::MAX);

        // Applies EIP-4788 pre-block contract call using `pre_block_beacon_root_contract_call`.
        pre_block_beacon_root_contract_call(
            &mut db,
            &self.evm_config,
            &chain_spec,
            &initialized_cfg,
            &initialized_block_env,
            block_number,
            attributes.payload_attributes.timestamp,
            attributes.payload_attributes.parent_beacon_block_root,
        )
        .map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to apply beacon root contract call for empty payload"
            );
            PayloadBuilderError::Internal(err.into())
        })?;

        // Commits withdrawals using `commit_withdrawals`.
        let WithdrawalsOutcome { withdrawals_root, withdrawals } = commit_withdrawals(
            &mut db,
            &chain_spec,
            attributes.payload_attributes.timestamp,
            attributes.payload_attributes.withdrawals.clone(),
        )
        .map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to commit withdrawals for empty payload"
            );
            err
        })?;

        // Merges all transitions into bundle state in `db`, applying withdrawal balance changes
        // and 4788 contract call.
        db.merge_transitions(BundleRetention::PlainState);

        // Calculates the state root.
        let bundle_state = db.take_bundle();
        let state_root = db.database.state_root(&bundle_state).map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to calculate state root for empty payload"
            );
            err
        })?;

        // Initializes optional fields for excess and used blob gas if Cancun fork is active.
        let mut excess_blob_gas = None;
        let mut blob_gas_used = None;

        if chain_spec.is_cancun_active_at_timestamp(attributes.payload_attributes.timestamp) {
            excess_blob_gas = if chain_spec.is_cancun_active_at_timestamp(parent_block.timestamp) {
                let parent_excess_blob_gas = parent_block.excess_blob_gas.unwrap_or_default();
                let parent_blob_gas_used = parent_block.blob_gas_used.unwrap_or_default();
                Some(calculate_excess_blob_gas(parent_excess_blob_gas, parent_blob_gas_used))
            } else {
                // for the first post-fork block, both parent.blob_gas_used and
                // parent.excess_blob_gas are evaluated as 0
                Some(calculate_excess_blob_gas(0, 0))
            };

            blob_gas_used = Some(0);
        }

        // Constructs a `Header` for the block with all necessary fields populated.
        let header = Header {
            parent_hash: parent_block.hash(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: initialized_block_env.coinbase,
            state_root,
            transactions_root: EMPTY_TRANSACTIONS,
            withdrawals_root,
            receipts_root: EMPTY_RECEIPTS,
            logs_bloom: Default::default(),
            timestamp: attributes.payload_attributes.timestamp,
            mix_hash: attributes.payload_attributes.prev_randao,
            nonce: BEACON_NONCE,
            base_fee_per_gas: Some(base_fee),
            number: parent_block.number + 1,
            gas_limit: block_gas_limit,
            difficulty: U256::ZERO,
            gas_used: 0,
            extra_data,
            blob_gas_used,
            excess_blob_gas,
            parent_beacon_block_root: attributes.payload_attributes.parent_beacon_block_root,
            requests_root: None,
        };

        // Constructs a `Block` using the constructed `Header` and empty body.
        let block = Block {
            header,
            body: vec![],
            ommers: vec![],
            withdrawals,
            requests: None,
        };

        // Seals the block using `seal_slow` to finalize block construction.
        let sealed_block = block.seal_slow();

        // Constructs and returns an `OptimismBuiltPayload` using the constructed block and other
        // attributes from `config`.
        Ok(OptimismBuiltPayload::new(
            attributes.payload_attributes.payload_id(),
            sealed_block,
            U256::ZERO,
            chain_spec,
            attributes,
        ))
    }
}

/// Implementation of the `PayloadBuilder` trait for `OptimismPayloadBuilder`.
impl<Pool, Client, EvmConfig> PayloadBuilder<Pool, Client> for OptimismPayloadBuilder<EvmConfig>
where
    Client: StateProviderFactory,
    Pool: TransactionPool,
    EvmConfig: ConfigureEvm,
{
    type Attributes = OptimismPayloadBuilderAttributes;
    type BuiltPayload = OptimismBuiltPayload;

    /// Attempts to build an Optimism payload using the provided arguments.
    ///
    /// This function calls `optimism_payload_builder` to construct the payload based on the
    /// configuration and pending block computation.
    fn try_build(
        &self,
        args: BuildArguments<Pool, Client, OptimismPayloadBuilderAttributes, OptimismBuiltPayload>,
    ) -> Result<BuildOutcome<OptimismBuiltPayload>, PayloadBuilderError> {
        optimism_payload_builder(self.evm_config.clone(), args, self.compute_pending_block)
    }

    /// Defines behavior when a payload is missing during the build process.
    ///
    /// For this implementation, it waits for the job already in progress (`AwaitInProgress`),
    /// as racing another job doesn't provide any benefit.
    fn on_missing_payload(
        &self,
        _args: BuildArguments<Pool, Client, OptimismPayloadBuilderAttributes, OptimismBuiltPayload>,
    ) -> MissingPayloadBehaviour<Self::BuiltPayload> {
        // we want to await the job that's already in progress because that should be returned as
        // is, there's no benefit in racing another job
        MissingPayloadBehaviour::AwaitInProgress
    }

    /// Builds an empty payload using provided client and configuration.
    ///
    /// This function constructs an empty payload using details from the provided `config`.
    /// It fetches necessary state information and applies specific operations such as
    /// EIP-4788 pre-block contract call and withdrawals handling.
    fn build_empty_payload(
        &self,
        client: &Client,
        config: PayloadConfig<Self::Attributes>,
    ) -> Result<OptimismBuiltPayload, PayloadBuilderError> {
        let extra_data = config.extra_data();
        let PayloadConfig {
            initialized_block_env,
            parent_block,
            attributes,
            chain_spec,
            initialized_cfg,
            ..
        } = config;

        // Debug logging: Logs information about building an empty payload.
        debug!(
            target: "payload_builder",
            parent_hash = ?parent_block.hash(),
            parent_number = parent_block.number,
            "building empty payload"
        );

        // Retrieves state from client using parent block hash.
        let state = client.state_by_block_hash(parent_block.hash()).map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to get state for empty payload"
            );
            err
        })?;

        // Constructs a state database (`db`) from the retrieved state.
        let mut db = State::builder()
            .with_database(StateProviderDatabase::new(state))
            .with_bundle_update()
            .build();

        // Converts `initialized_block_env` fields into appropriate types.
        let base_fee = initialized_block_env.basefee.to::<u64>();
        let block_number = initialized_block_env.number.to::<u64>();
        let block_gas_limit: u64 = initialized_block_env.gas_limit.try_into().unwrap_or(u64::MAX);

        // Applies EIP-4788 pre-block contract call using `pre_block_beacon_root_contract_call`.
        pre_block_beacon_root_contract_call(
            &mut db,
            &self.evm_config,
            &chain_spec,
            &initialized_cfg,
            &initialized_block_env,
            block_number,
            attributes.payload_attributes.timestamp,
            attributes.payload_attributes.parent_beacon_block_root,
        )
        .map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to apply beacon root contract call for empty payload"
            );
            PayloadBuilderError::Internal(err.into())
        })?;

        // Commits withdrawals using `commit_withdrawals`.
        let WithdrawalsOutcome { withdrawals_root, withdrawals } = commit_withdrawals(
            &mut db,
            &chain_spec,
            attributes.payload_attributes.timestamp,
            attributes.payload_attributes.withdrawals.clone(),
        )
        .map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to commit withdrawals for empty payload"
            );
            err
        })?;

        // Merges all transitions into bundle state in `db`, applying withdrawal balance changes
        // and 4788 contract call.
        db.merge_transitions(BundleRetention::PlainState);

        // Calculates the state root.
        let bundle_state = db.take_bundle();
        let state_root = db.database.state_root(&bundle_state).map_err(|err| {
            warn!(
                target: "payload_builder",
                parent_hash = %parent_block.hash(),
                %err,
                "failed to calculate state root for empty payload"
            );
            err
        })?;

        // Initializes optional fields for excess and used blob gas if Cancun fork is active.
        let mut excess_blob_gas = None;
        let mut blob_gas_used = None;

        if chain_spec.is_cancun_active_at_timestamp(attributes.payload_attributes.timestamp) {
            excess_blob_gas = if chain_spec.is_cancun_active_at_timestamp(parent_block.timestamp) {
                let parent_excess_blob_gas = parent_block.excess_blob_gas.unwrap_or_default();
                let parent_blob_gas_used = parent_block.blob_gas_used.unwrap_or_default();
                Some(calculate_excess_blob_gas(parent_excess_blob_gas, parent_blob_gas_used))
            } else {
                // for the first post-fork block, both parent.blob_gas_used and
                // parent.excess_blob_gas are evaluated as 0
                Some(calculate_excess_blob_gas(0, 0))
            };

            blob_gas_used = Some(0);
        }

        // Constructs a `Header` for the block with all necessary fields populated.
        let header = Header {
            parent_hash: parent_block.hash(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: initialized_block_env.coinbase,
            state_root,
            transactions_root: EMPTY_TRANSACTIONS,
            withdrawals_root,
            receipts_root: EMPTY_RECEIPTS,
            logs_bloom: Default::default(),
            timestamp: attributes.payload_attributes.timestamp,
            mix_hash: attributes.payload_attributes.prev_randao,
            nonce: BEACON_NONCE,
            base_fee_per_gas: Some(base_fee),
            number: parent_block.number + 1,
            gas_limit: block_gas_limit,
            difficulty: U256::ZERO,
            gas_used: 0,
            extra_data,
            blob_gas_used,
            excess_blob_gas,
            parent_beacon_block_root: attributes.payload_attributes.parent_beacon_block_root,
            requests_root: None,
        };

        // Constructs a `Block` using the constructed `Header` and empty body.
        let block = Block {
            header,
            body: vec![],
            ommers: vec![],
            withdrawals,
            requests: None,
        };

        // Seals the block using `seal_slow` to finalize block construction.
        let sealed_block = block.seal_slow();

        // Constructs and returns an `OptimismBuiltPayload` using the constructed block and other
        // attributes from `config`.
        Ok(OptimismBuiltPayload::new(
            attributes.payload_attributes.payload_id(),
            sealed_block,
            U256::ZERO,
            chain_spec,
            attributes,
        ))
    }
}

/// Constructs an Ethereum transaction payload from the transactions sent through the
/// Payload attributes by the sequencer. If the `no_tx_pool` argument is passed in
/// the payload attributes, the transaction pool will be ignored and the only transactions
/// included in the payload will be those sent through the attributes.
///
/// Given build arguments including an Ethereum client, transaction pool,
/// and configuration, this function creates a transaction payload. Returns
/// a result indicating success with the payload or an error in case of failure.
#[inline]
pub(crate) fn optimism_payload_builder<EvmConfig, Pool, Client>(
    evm_config: EvmConfig,
    args: BuildArguments<Pool, Client, OptimismPayloadBuilderAttributes, OptimismBuiltPayload>,
    _compute_pending_block: bool,
) -> Result<BuildOutcome<OptimismBuiltPayload>, PayloadBuilderError>
where
    EvmConfig: ConfigureEvm,
    Client: StateProviderFactory,
    Pool: TransactionPool,
{
    // Destructure arguments from the provided BuildArguments structure
    let BuildArguments { client, pool, mut cached_reads, config, cancel, best_payload } = args;

    // Retrieve state provider from the client using the parent block's hash
    let state_provider = client.state_by_block_hash(config.parent_block.hash())?;
    let state = StateProviderDatabase::new(state_provider);

    // Initialize the EVM state with the retrieved state provider and cached reads
    let mut db =
        State::builder().with_database_ref(cached_reads.as_db(state)).with_bundle_update().build();

    // Extract necessary configuration and attributes from the PayloadConfig
    let extra_data = config.extra_data();
    let PayloadConfig {
        initialized_block_env,
        initialized_cfg,
        parent_block,
        attributes,
        chain_spec,
        ..
    } = config;

    // Log debug information about the payload being built
    debug!(target: "payload_builder", id=%attributes.payload_attributes.payload_id(), parent_hash = ?parent_block.hash(), parent_number = parent_block.number, "building new payload");

    // Initialize variables for gas limit, base fee, and executed transactions
    let mut cumulative_gas_used = 0;
    let block_gas_limit: u64 = attributes
        .gas_limit
        .unwrap_or_else(|| initialized_block_env.gas_limit.try_into().unwrap_or(u64::MAX));
    let base_fee = initialized_block_env.basefee.to::<u64>();

    let mut executed_txs = Vec::with_capacity(attributes.transactions.len());

    // Retrieve best transactions from the pool based on provided attributes
    let mut best_txs = pool.best_transactions_with_attributes(BestTransactionsAttributes::new(
        base_fee,
        initialized_block_env.get_blob_gasprice().map(|gasprice| gasprice as u64),
    ));

    let mut total_fees = U256::ZERO;

    let block_number = initialized_block_env.number.to::<u64>();

    // Check if Regolith hardfork is active to handle specific behaviors
    let is_regolith = chain_spec.is_fork_active_at_timestamp(
        OptimismHardfork::Regolith,
        attributes.payload_attributes.timestamp,
    );

    // apply eip-4788 pre block contract call
    pre_block_beacon_root_contract_call(
        &mut db,
        &evm_config,
        &chain_spec,
        &initialized_cfg,
        &initialized_block_env,
        block_number,
        attributes.payload_attributes.timestamp,
        attributes.payload_attributes.parent_beacon_block_root,
    )
    .map_err(|err| {
        // Log warning if applying beacon root contract call fails
        warn!(target: "payload_builder",
            parent_hash=%parent_block.hash(),
            %err,
            "failed to apply beacon root contract call for empty payload"
        );
        // Return internal error encapsulated in PayloadBuilderError
        PayloadBuilderError::Internal(err.into())
    })?;

    // Ensure that the create2deployer is force-deployed at the canyon transition. Optimism
    // blocks will always have at least a single transaction in them (the L1 info transaction),
    // so we can safely assume that this will always be triggered upon the transition and that
    // the above check for empty blocks will never be hit on OP chains.
    reth_evm_optimism::ensure_create2_deployer(
        chain_spec.clone(),
        attributes.payload_attributes.timestamp,
        &mut db,
    )
    .map_err(|err| {
        warn!(target: "payload_builder", %err, "missing create2 deployer, skipping block.");
        PayloadBuilderError::other(OptimismPayloadBuilderError::ForceCreate2DeployerFail)
    })?;

    // Initialize receipts vector to store transaction receipts
    let mut receipts = Vec::with_capacity(attributes.transactions.len());

    // Process each sequencer transaction in the attributes
    for sequencer_tx in &attributes.transactions {
        // Check if the job was cancelled, if so we can exit early.
        if cancel.is_cancelled() {
            return Ok(BuildOutcome::Cancelled)
        }

        // A sequencer's block should never contain blob transactions.
        if sequencer_tx.is_eip4844() {
            return Err(PayloadBuilderError::other(
                OptimismPayloadBuilderError::BlobTransactionRejected,
            ))
        }

        // Convert the transaction to a [TransactionSignedEcRecovered]. This is
        // purely for the purposes of utilizing the `evm_config.tx_env`` function.
        // Deposit transactions do not have signatures, so if the tx is a deposit, this
        // will just pull in its `from` address.
        let sequencer_tx = sequencer_tx.clone().try_into_ecrecovered().map_err(|_| {
            PayloadBuilderError::other(OptimismPayloadBuilderError::TransactionEcRecoverFailed)
        })?;

        // Cache the depositor account prior to the state transition for the deposit nonce.
        //
        // Note that this *only* needs to be done post-regolith hardfork, as deposit nonces
        // were not introduced in Bedrock. In addition, regular transactions don't have deposit
        // nonces, so we don't need to touch the DB for those.
        let depositor = (is_regolith && sequencer_tx.is_deposit())
            .then(|| {
                db.load_cache_account(sequencer_tx.signer())
                    .map(|acc| acc.account_info().unwrap_or_default())
            })
            .transpose()
            .map_err(|_| {
                PayloadBuilderError::other(OptimismPayloadBuilderError::AccountLoadFailed(
                    sequencer_tx.signer(),
                ))
            })?;

        // Create the execution environment for the transaction
        let env = EnvWithHandlerCfg::new_with_cfg_env(
            initialized_cfg.clone(),
            initialized_block_env.clone(),
            evm_config.tx_env(&sequencer_tx),
        );

        // Execute the transaction on the EVM
        let mut evm = evm_config.evm_with_env(&mut db, env);

        let ResultAndState { result, state } = match evm.transact() {
            Ok(res) => res,
            Err(err) => {
                match err {
                    EVMError::Transaction(err) => {
                        trace!(target: "payload_builder", %err, ?sequencer_tx, "Error in sequencer transaction, skipping.");
                        continue
                    }
                    err => {
                        // this is an error that we should treat as fatal for this attempt
                        return Err(PayloadBuilderError::EvmExecutionError(err))
                    }
                }
            }
        };

        // Release the EVM reference to the database and commit the changes
        drop(evm);
        db.commit(state);

        let gas_used = result.gas_used();

        // Add gas used by the transaction to cumulative gas used, before creating the receipt
        cumulative_gas_used += gas_used;

        // Create and store the transaction receipt
        receipts.push(Some(Receipt {
            tx_type: sequencer_tx.tx_type(),
            success: result.is_success(),
            cumulative_gas_used,
            logs: result.into_logs().into_iter().map(Into::into).collect(),
            deposit_nonce: depositor.map(|account| account.nonce),
            // The deposit receipt version was introduced in Canyon to indicate an update to how
            // receipt hashes should be computed when set. The state transition process
            // ensures this is only set for post-Canyon deposit transactions.
            deposit_receipt_version: chain_spec
                .is_fork_active_at_timestamp(
                    OptimismHardfork::Canyon,
                    attributes.payload_attributes.timestamp,
                )
                .then_some(1),
        }));

        // Append transaction to the list of executed transactions
        executed_txs.push(sequencer_tx.into_signed());
    }

    // Process transactions from the transaction pool if `no_tx_pool` is false
    if !attributes.no_tx_pool {
        while let Some(pool_tx) = best_txs.next() {
            // Ensure we still have capacity for this transaction
            if cumulative_gas_used + pool_tx.gas_limit() > block_gas_limit {
                // We can't fit this transaction into the block, so we need to mark it as
                // invalid which also removes all dependent transaction from
                // the iterator before we can continue
                best_txs.mark_invalid(&pool_tx);
                continue
            }

            // A sequencer's block should never contain blob or deposit transactions from the pool.
            if pool_tx.is_eip4844() || pool_tx.tx_type() == TxType::Deposit as u8 {
                best_txs.mark_invalid(&pool_tx)
            }

            // Check if the job was cancelled, if so we can exit early
            if cancel.is_cancelled() {
                return Ok(BuildOutcome::Cancelled)
            }

            // Convert transaction to a signed transaction
            let tx = pool_tx.to_recovered_transaction();
            let env = EnvWithHandlerCfg::new_with_cfg_env(
                initialized_cfg.clone(),
                initialized_block_env.clone(),
                evm_config.tx_env(&tx),
            );

            // Configure the environment for the block
            let mut evm = evm_config.evm_with_env(&mut db, env);

            let ResultAndState { result, state } = match evm.transact() {
                Ok(res) => res,
                Err(err) => {
                    match err {
                        EVMError::Transaction(err) => {
                            if matches!(err, InvalidTransaction::NonceTooLow { .. }) {
                                // If the nonce is too low, we can skip this transaction
                                trace!(target: "payload_builder", %err, ?tx, "skipping nonce too low transaction");
                            } else {
                                // If the transaction is invalid, we can skip it and all of its
                                // descendants
                                trace!(target: "payload_builder", %err, ?tx, "skipping invalid transaction and its descendants");
                                best_txs.mark_invalid(&pool_tx);
                            }

                            continue
                        }
                        err => {
                            // This is an error that we should treat as fatal for this attempt
                            return Err(PayloadBuilderError::EvmExecutionError(err))
                        }
                    }
                }
            };

            // Release the EVM reference to the database and commit the changes
            drop(evm);
            db.commit(state);

            let gas_used = result.gas_used();

            // Add gas used by the transaction to cumulative gas used, before creating the
            // receipt
            cumulative_gas_used += gas_used;

            // Create and store the transaction receipt
            receipts.push(Some(Receipt {
                tx_type: tx.tx_type(),
                success: result.is_success(),
                cumulative_gas_used,
                logs: result.into_logs().into_iter().map(Into::into).collect(),
                deposit_nonce: None,
                deposit_receipt_version: None,
            }));

            // Update total fees with the miner fee for the transaction
            let miner_fee = tx
                .effective_tip_per_gas(Some(base_fee))
                .expect("fee is always valid; execution succeeded");
            total_fees += U256::from(miner_fee) * U256::from(gas_used);

            // Append transaction to the list of executed transactions
            executed_txs.push(tx.into_signed());
        }
    }

    // Check if we have a better block compared to the best payload
    if !is_better_payload(best_payload.as_ref(), total_fees) {
        // Skip building the block if the current payload is not better
        return Ok(BuildOutcome::Aborted { fees: total_fees, cached_reads })
    }

    // Commit withdrawals and obtain withdrawal root and list
    let WithdrawalsOutcome { withdrawals_root, withdrawals } = commit_withdrawals(
        &mut db,
        &chain_spec,
        attributes.payload_attributes.timestamp,
        attributes.clone().payload_attributes.withdrawals,
    )?;

    // Merge all transitions into bundle state, applying withdrawal balance changes
    // and 4788 contract call
    db.merge_transitions(BundleRetention::PlainState);

    // Create execution outcome containing bundle and receipts
    let execution_outcome =
        ExecutionOutcome::new(db.take_bundle(), vec![receipts].into(), block_number, Vec::new());
    // Calculate receipts root using optimism_receipts_root_slow method
    let receipts_root = execution_outcome
        .optimism_receipts_root_slow(
            block_number,
            chain_spec.as_ref(),
            attributes.payload_attributes.timestamp,
        )
        .expect("Number is in range");
    // Calculate logs bloom filter for the block
    let logs_bloom = execution_outcome.block_logs_bloom(block_number).expect("Number is in range");

    // Calculate state root
    let state_root = {
        let state_provider = db.database.0.inner.borrow_mut();
        state_provider.db.state_root(execution_outcome.state())?
    };

    // Calculate transactions root using calculate_transaction_root function
    let transactions_root = proofs::calculate_transaction_root(&executed_txs);

    // Initialize empty blob sidecars. There are no blob transactions on L2.
    let blob_sidecars = Vec::new();
    let mut excess_blob_gas = None;
    let mut blob_gas_used = None;

    // Determine cancun fields when active
    if chain_spec.is_cancun_active_at_timestamp(attributes.payload_attributes.timestamp) {
        excess_blob_gas = if chain_spec.is_cancun_active_at_timestamp(parent_block.timestamp) {
            let parent_excess_blob_gas = parent_block.excess_blob_gas.unwrap_or_default();
            let parent_blob_gas_used = parent_block.blob_gas_used.unwrap_or_default();
            Some(calculate_excess_blob_gas(parent_excess_blob_gas, parent_blob_gas_used))
        } else {
            // For the first post-fork block, both parent.blob_gas_used and
            // parent.excess_blob_gas are evaluated as 0
            Some(calculate_excess_blob_gas(0, 0))
        };

        blob_gas_used = Some(0);
    }

    // Create the block header
    let header = Header {
        parent_hash: parent_block.hash(),
        ommers_hash: EMPTY_OMMER_ROOT_HASH,
        beneficiary: initialized_block_env.coinbase,
        state_root,
        transactions_root,
        receipts_root,
        withdrawals_root,
        logs_bloom,
        timestamp: attributes.payload_attributes.timestamp,
        mix_hash: attributes.payload_attributes.prev_randao,
        nonce: BEACON_NONCE,
        base_fee_per_gas: Some(base_fee),
        number: parent_block.number + 1,
        gas_limit: block_gas_limit,
        difficulty: U256::ZERO,
        gas_used: cumulative_gas_used,
        extra_data,
        parent_beacon_block_root: attributes.payload_attributes.parent_beacon_block_root,
        blob_gas_used,
        excess_blob_gas,
        requests_root: None,
    };

    // Create the block with header, executed transactions, withdrawals, and requests
    let block = Block { header, body: executed_txs, ommers: vec![], withdrawals, requests: None };

    // Seal the block and obtain the sealed block
    let sealed_block = block.seal_slow();
    debug!(target: "payload_builder", ?sealed_block, "sealed built block");

    // Create OptimismBuiltPayload containing the sealed block, total fees, chain spec, and attributes
    let mut payload = OptimismBuiltPayload::new(
        attributes.payload_attributes.id,
        sealed_block,
        total_fees,
        chain_spec,
        attributes,
    );

    // Extend the payload with blob sidecars from the executed transactions
    payload.extend_sidecars(blob_sidecars);

    // Return the successful outcome with the built payload and cached reads
    Ok(BuildOutcome::Better { payload, cached_reads })
}
