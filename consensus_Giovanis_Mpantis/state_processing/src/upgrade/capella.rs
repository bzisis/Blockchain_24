use std::mem;
use types::{
    BeaconState, BeaconStateCapella, BeaconStateError as Error, ChainSpec, EpochCache, EthSpec,
    Fork, List,
};

/// Upgrade a `Bellatrix` state to a `Capella` state.
///
/// This function transforms a given `Bellatrix` state into its corresponding `Capella` state,
/// following the specified chain specification.
///
/// # Arguments
///
/// * `pre_state` - Mutable reference to the `Bellatrix` state to be upgraded.
/// * `spec` - Chain specification defining the parameters of the upgrade.
///
/// # Errors
///
/// Returns an error if any required fields are missing or if the transformation fails.
///
/// # Examples
///
/// ```rust
/// use types::{BeaconState, ChainSpec};
/// use your_module::upgrade_to_capella;
///
/// let mut state: BeaconState<Bellatrix> = /* initialize Bellatrix state */;
/// let spec: ChainSpec = /* initialize ChainSpec */;
///
/// match upgrade_to_capella(&mut state, &spec) {
///     Ok(_) => println!("State upgraded successfully to Capella."),
///     Err(e) => eprintln!("Error upgrading state: {:?}", e),
/// }
/// ```
pub fn upgrade_to_capella<E: EthSpec>(
    pre_state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let epoch = pre_state.current_epoch();
    let pre = pre_state.as_bellatrix_mut()?;

    // Where possible, use something like `mem::take` to move fields from behind the &mut
    // reference. For other fields that don't have a good default value, use `clone`.
    //
    // Fixed size vectors get cloned because replacing them would require the same size
    // allocation as cloning.
    let post = BeaconState::Capella(BeaconStateCapella {
        // Versioning
        genesis_time: pre.genesis_time,
        genesis_validators_root: pre.genesis_validators_root,
        slot: pre.slot,
        fork: Fork {
            previous_version: pre.fork.current_version,
            current_version: spec.capella_fork_version,
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
        latest_execution_payload_header: pre.latest_execution_payload_header.upgrade_to_capella(),
        // Capella-specific fields
        next_withdrawal_index: 0,
        next_withdrawal_validator_index: 0,
        historical_summaries: List::default(),
        // Caches
        total_active_balance: pre.total_active_balance,
        progressive_balances_cache: mem::take(&mut pre.progressive_balances_cache),
        committee_caches: mem::take(&mut pre.committee_caches),
        pubkey_cache: mem::take(&mut pre.pubkey_cache),
        exit_cache: mem::take(&mut pre.exit_cache),
        slashings_cache: mem::take(&mut pre.slashings_cache),
        epoch_cache: EpochCache::default(),
    });

    *pre_state = post;

    Ok(())
}
