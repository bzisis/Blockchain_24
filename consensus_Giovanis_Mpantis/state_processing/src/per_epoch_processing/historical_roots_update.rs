use super::errors::EpochProcessingError;
use safe_arith::SafeArith;
use tree_hash::TreeHash;
use types::beacon_state::BeaconState;
use types::eth_spec::EthSpec;
use types::Unsigned;

/// Updates the historical roots of the given `BeaconState`.
///
/// This function checks if the `next_epoch` of the state is a multiple of
/// `SlotsPerHistoricalRoot / slots_per_epoch`. If so, it computes the
/// historical batch root using the `tree_hash_root` function and pushes
/// it to the `historical_roots` of the state.
///
/// # Arguments
///
/// * `state` - A mutable reference to the `BeaconState`.
///
/// # Returns
///
/// * `Ok(())` - If the historical roots update is processed successfully.
/// * `Err(EpochProcessingError)` - If there is an error during the processing.
///
/// # Errors
///
/// This function will return an error if:
/// - The computation of the next epoch fails.
/// - The computation of the remainder operation fails.
/// - The historical batch retrieval or tree hash root computation fails.
/// - Pushing the historical root to the state's historical roots fails.
///
/// # Examples
///
/// ```
/// use types::eth_spec::MainnetEthSpec;
/// use types::beacon_state::BeaconState;
/// use crate::process_historical_roots_update;
///
/// let mut state = BeaconState::<MainnetEthSpec>::default();
/// let result = process_historical_roots_update(&mut state);
/// assert!(result.is_ok());
/// ```
///
/// # Note
///
/// This function assumes that the `BeaconState` struct and its methods
/// (`next_epoch`, `historical_batch`, `historical_roots_mut`, etc.) are
/// correctly implemented and that the `safe_arith` and `tree_hash` crates
/// are available.
pub fn process_historical_roots_update<E: EthSpec>(
    state: &mut BeaconState<E>,
) -> Result<(), EpochProcessingError> {
    let next_epoch = state.next_epoch()?;
    if next_epoch
        .as_u64()
        .safe_rem(E::SlotsPerHistoricalRoot::to_u64().safe_div(E::slots_per_epoch())?)?
        == 0
    {
        let historical_batch = state.historical_batch()?;
        state
            .historical_roots_mut()
            .push(historical_batch.tree_hash_root())?;
    }
    Ok(())
}
