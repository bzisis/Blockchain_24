use crate::EpochProcessingError;
use safe_arith::SafeArith;
use std::sync::Arc;
use types::beacon_state::BeaconState;
use types::chain_spec::ChainSpec;
use types::eth_spec::EthSpec;

/// Processes updates related to the sync committee for the next epoch.
///
/// Updates the current sync committee and prepares the next sync committee in the given `BeaconState`.
///
/// # Arguments
///
/// * `state` - The mutable reference to the `BeaconState` where sync committee updates will be processed.
/// * `spec` - The `ChainSpec` configuration for the beacon chain.
///
/// # Errors
///
/// Returns an `EpochProcessingError` if there is an issue during epoch processing.
///
/// # Behavior
///
/// This function checks if the next epoch is a multiple of `epochs_per_sync_committee_period` specified in `spec`.
/// If true, it updates the current sync committee to the next sync committee and prepares the subsequent sync committee.
///
/// # Returns
///
/// Returns `Ok(())` if sync committee updates are processed successfully.
///
pub fn process_sync_committee_updates<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), EpochProcessingError> {
    // Determine the next epoch after the current epoch.
    let next_epoch = state.next_epoch()?;
    
    // Check if the next epoch is a multiple of epochs_per_sync_committee_period.
    if next_epoch.safe_rem(spec.epochs_per_sync_committee_period)? == 0 {
        // Update the current sync committee to the next sync committee.
        *state.current_sync_committee_mut()? = state.next_sync_committee()?.clone();

        // Prepare the next sync committee based on the state and chain spec.
        *state.next_sync_committee_mut()? = Arc::new(state.get_next_sync_committee(spec)?);
    }
    
    // Return Ok indicating successful processing of sync committee updates.
    Ok(())
}
