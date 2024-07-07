/// This module contains functions and structures for processing various parts of the epoch
/// transition in the Beacon Chain.
use super::{process_registry_updates, process_slashings, EpochProcessingSummary, Error};
use crate::epoch_cache::initialize_epoch_cache;
use crate::per_epoch_processing::{
    effective_balance_updates::process_effective_balance_updates,
    historical_roots_update::process_historical_roots_update,
    resets::{process_eth1_data_reset, process_randao_mixes_reset, process_slashings_reset},
};
pub use justification_and_finalization::process_justification_and_finalization;
pub use participation_record_updates::process_participation_record_updates;
pub use rewards_and_penalties::process_rewards_and_penalties;
use types::{BeaconState, ChainSpec, EthSpec, RelativeEpoch};
pub use validator_statuses::{TotalBalances, ValidatorStatus, ValidatorStatuses};

pub mod justification_and_finalization;
pub mod participation_record_updates;
pub mod rewards_and_penalties;
pub mod validator_statuses;

/// Processes the epoch transition for a given `BeaconState`.
///
/// This function performs all the required operations to transition the beacon chain state
/// from one epoch to the next. It handles updates to various caches, validator statuses, 
/// balances, slashings, and more.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `BeaconState` to be updated.
/// * `spec` - A reference to the `ChainSpec` containing constants used in the processing.
///
/// # Returns
///
/// * `Result<EpochProcessingSummary<E>, Error>` - A summary of the epoch processing or an error
///   if something went wrong.
///
/// # Errors
///
/// This function can return various errors if any of the processing steps fail.
pub fn process_epoch<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<EpochProcessingSummary<E>, Error> {
    // Ensure the committee caches are built.
    state.build_committee_cache(RelativeEpoch::Previous, spec)?;
    state.build_committee_cache(RelativeEpoch::Current, spec)?;
    state.build_committee_cache(RelativeEpoch::Next, spec)?;
    state.build_total_active_balance_cache(spec)?;
    initialize_epoch_cache(state, spec)?;

    // Load the struct we use to assign validators into sets based on their participation.
    //
    // E.g., attestation in the previous epoch, attested to the head, etc.
    let mut validator_statuses = ValidatorStatuses::new(state, spec)?;
    validator_statuses.process_attestations(state)?;

    // Justification and finalization.
    let justification_and_finalization_state =
        process_justification_and_finalization(state, &validator_statuses.total_balances, spec)?;
    justification_and_finalization_state.apply_changes_to_state(state);

    // Rewards and Penalties.
    process_rewards_and_penalties(state, &validator_statuses, spec)?;

    // Registry Updates.
    process_registry_updates(state, spec)?;

    // Slashings.
    process_slashings(
        state,
        validator_statuses.total_balances.current_epoch(),
        spec,
    )?;

    // Reset eth1 data votes.
    process_eth1_data_reset(state)?;

    // Update effective balances with hysteresis (lag).
    process_effective_balance_updates(state, spec)?;

    // Reset slashings
    process_slashings_reset(state)?;

    // Set randao mix
    process_randao_mixes_reset(state)?;

    // Set historical root accumulator
    process_historical_roots_update(state)?;

    // Rotate current/previous epoch attestations
    process_participation_record_updates(state)?;

    // Rotate the epoch caches to suit the epoch transition.
    state.advance_caches()?;

    Ok(EpochProcessingSummary::Base {
        total_balances: validator_statuses.total_balances,
        statuses: validator_statuses.statuses,
    })
}
