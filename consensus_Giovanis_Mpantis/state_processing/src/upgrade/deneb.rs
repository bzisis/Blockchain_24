use std::mem;
use types::{
    BeaconState, BeaconStateDeneb, BeaconStateError as Error, ChainSpec, EpochCache, EthSpec, Fork,
};

/// Upgrade a `Capella` state to a `Deneb` state.
///
/// This function transforms a `Capella` state into an `Deneb` state by reorganizing and
/// transforming various fields as necessary.
///
/// # Arguments
///
/// * `pre_state` - The mutable reference to the current `Capella` state that will be upgraded.
/// * `spec` - The chain specification that includes the `Deneb` fork version and other parameters.
///
/// # Errors
///
/// Returns `Ok(())` if the upgrade is successful, or an `Error` indicating the reason for failure.
///
/// # Notes
///
/// This function carefully moves and clones fields from the `Capella` state to the `Deneb` state,
/// ensuring compatibility and maintaining data integrity during the upgrade process.
pub fn upgrade_to_deneb<E: EthSpec>(
    pre_state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let epoch = pre_state.current_epoch();
    let pre = pre_state.as_capella_mut()?;

    let previous_fork_version = pre.fork.current_version;

    // Create a new `BeaconStateDeneb` from the `Capella` state's fields
    let post = BeaconState::Deneb(BeaconStateDeneb {
        // Versioning
        genesis_time: pre.genesis_time,
        genesis_validators_root: pre.genesis_validators_root,
        slot: pre.slot,
        fork: Fork {
            previous_version: previous_fork_version,
            current_version: spec.deneb_fork_version,
            epoch,
        },
        // History
        latest_block_header: pre.latest_block_header.clone(),
        block_roots: pre.block_roots.clone(),
        state_roots: pre.state_roots.clone(),
        historical_roots: mem::take(&mut pre.historical_roots),
        // Eth1
        eth1_data: pre.eth1_data.clone(),
        eth1_data_votes: mem::take(&mut pre.eth1_data_votes),
        eth1_deposit_index: pre.eth1_deposit_index,
        // Registry
        validators: mem::take(&mut pre.validators),
        balances: mem::take(&mut pre.balances),
        // Randomness
        randao_mixes: pre.randao_mixes.clone(),
        // Slashings
        slashings: pre.slashings.clone(),
        // Participation
        previous_epoch_participation: mem::take(&mut pre.previous_epoch_participation),
        current_epoch_participation: mem::take(&mut pre.current_epoch_participation),
        // Finality
        justification_bits: pre.justification_bits.clone(),
        previous_justified_checkpoint: pre.previous_justified_checkpoint,
        current_justified_checkpoint: pre.current_justified_checkpoint,
        finalized_checkpoint: pre.finalized_checkpoint,
        // Inactivity
        inactivity_scores: mem::take(&mut pre.inactivity_scores),
        // Sync committees
        current_sync_committee: pre.current_sync_committee.clone(),
        next_sync_committee: pre.next_sync_committee.clone(),
        // Execution
        latest_execution_payload_header: pre.latest_execution_payload_header.upgrade_to_deneb(),
        // Capella
        next_withdrawal_index: pre.next_withdrawal_index,
        next_withdrawal_validator_index: pre.next_withdrawal_validator_index,
        historical_summaries: pre.historical_summaries.clone(),
        // Caches
        total_active_balance: pre.total_active_balance,
        progressive_balances_cache: mem::take(&mut pre.progressive_balances_cache),
        committee_caches: mem::take(&mut pre.committee_caches),
        pubkey_cache: mem::take(&mut pre.pubkey_cache),
        exit_cache: mem::take(&mut pre.exit_cache),
        slashings_cache: mem::take(&mut pre.slashings_cache),
        epoch_cache: EpochCache::default(),
    });

    // Replace the original `pre_state` with the upgraded `Deneb` state
    *pre_state = post;

    Ok(())
}
