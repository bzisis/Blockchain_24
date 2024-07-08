//! Error type

/// Optimism specific payload building errors.
#[derive(Debug, thiserror::Error)]
pub enum OptimismPayloadBuilderError {
    /// Thrown when a transaction fails to convert to a
    /// [`reth_primitives::TransactionSignedEcRecovered`].
    #[error("failed to convert deposit transaction to TransactionSignedEcRecovered")]
    TransactionEcRecoverFailed,
    /// Thrown when the L1 block info could not be parsed from the calldata of the
    /// first transaction supplied in the payload attributes.
    #[error("failed to parse L1 block info from L1 info tx calldata")]
    L1BlockInfoParseFailed,
    /// Thrown when a database account could not be loaded.
    #[error("failed to load account {0}")]
    AccountLoadFailed(reth_primitives::Address),
    /// Thrown when force deploy of create2deployer code fails.
    #[error("failed to force create2deployer account code")]
    ForceCreate2DeployerFail,
    /// Thrown when a blob transaction is included in a sequencer's block.
    #[error("blob transaction included in sequencer block")]
    BlobTransactionRejected,
}

// More comments added as requested:

/// Ensure that the create2deployer is force-deployed at the canyon transition. Optimism
/// blocks will always have at least a single transaction in them (the L1 info transaction),
/// so we can safely assume that this will always be triggered upon the transition and that
/// the above check for empty blocks will never be hit on OP chains.
reth_evm_optimism::ensure_create2_deployer(
    chain_spec.clone(),
    attributes.payload_attributes.timestamp,
    &mut db,
)
.map_err(|err| {
    // Log a warning and handle the error if create2 deployer deployment fails.
    warn!(target: "payload_builder", %err, "missing create2 deployer, skipping block.");
    PayloadBuilderError::other(OptimismPayloadBuilderError::ForceCreate2DeployerFail)
})?;

let mut receipts = Vec::with_capacity(attributes.transactions.len());
for sequencer_tx in &attributes.transactions {
    // Check if the job was cancelled, if so we can exit early.
    if cancel.is_cancelled() {
        return Ok(BuildOutcome::Cancelled);
    }

    // A sequencer's block should never contain blob transactions.
    if sequencer_tx.is_eip4844() {
        return Err(PayloadBuilderError::other(
            OptimismPayloadBuilderError::BlobTransactionRejected,
        ));
    }

    // Convert the transaction to a [TransactionSignedEcRecovered]. This is
    // purely for the purposes of utilizing the `evm_config.tx_env` function.
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

    // Prepare the execution environment with configuration and transaction data.
    let env = EnvWithHandlerCfg::new_with_cfg_env(
        initialized_cfg.clone(),
        initialized_block_env.clone(),
        evm_config.tx_env(&sequencer_tx),
    );

    // Initialize the EVM with the prepared environment and database.
    let mut evm = evm_config.evm_with_env(&mut db, env);

    // Execute the transaction within the EVM and handle any errors.
    let ResultAndState { result, state } = match evm.transact() {
        Ok(res) => res,
        Err(err) => {
            match err {
                EVMError::Transaction(err) => {
                    // Log a trace message for non-fatal transaction errors and continue processing.
                    trace!(target: "payload_builder", %err, ?sequencer_tx, "Error in sequencer transaction, skipping.");
                    continue;
                }
                err => {
                    // Handle fatal EVM errors that require stopping further processing.
                    return Err(PayloadBuilderError::EvmExecutionError(err));
                }
            }
        }
    };

    // Release the EVM reference to ensure no DB locks and commit changes to DB.
    drop(evm);
    db.commit(state);

    // Track cumulative gas usage for the block.
    let gas_used = result.gas_used();
    cumulative_gas_used += gas_used;

    // Construct the receipt for the executed transaction.
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

    // Append the executed transaction to the list of processed transactions.
    executed_txs.push(sequencer_tx.into_signed());
}
