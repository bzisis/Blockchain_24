use crate::EpochProcessingError;
use types::beacon_state::BeaconState;
use types::eth_spec::EthSpec;

/// Process participation record updates for the given BeaconState.
///
/// This function moves attestations from the current epoch to the previous epoch within
/// the given BeaconState. It assumes that the BeaconState is mutable and modifies it
/// in place.
///
/// # Arguments
///
/// * `state` - A mutable reference to a BeaconState object.
///
/// # Errors
///
/// Returns an `EpochProcessingError` if there are any errors encountered during the
/// processing of participation record updates.
///
/// # Example
///
/// ```rust
/// use types::{BeaconState, EthSpec};
/// use crate::process_participation_record_updates;
///
/// fn main() {
///     let mut state: BeaconState<MyEthSpec> = BeaconState::default();
///     match process_participation_record_updates(&mut state) {
///         Ok(()) => println!("Participation record updates processed successfully"),
///         Err(e) => eprintln!("Error processing participation record updates: {:?}", e),
///     }
/// }
/// ```
pub fn process_participation_record_updates<E: EthSpec>(
    state: &mut BeaconState<E>,
) -> Result<(), EpochProcessingError> {
    let base_state = state.as_base_mut()?;
    base_state.previous_epoch_attestations =
        std::mem::take(&mut base_state.current_epoch_attestations);
    Ok(())
}
