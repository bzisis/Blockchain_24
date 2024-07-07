use crate::EpochProcessingError;
use types::beacon_state::BeaconState;
use types::eth_spec::EthSpec;
use types::participation_flags::ParticipationFlags;
use types::List;

/// Process participation flag updates for the given beacon state.
///
/// Updates the participation flags for the previous and current epochs based on the
/// current state of validators.
///
/// # Arguments
///
/// * `state` - A mutable reference to a `BeaconState` instance.
///
/// # Errors
///
/// Returns an `EpochProcessingError` if there is an issue updating the participation flags.
///
/// # Returns
///
/// Returns `Ok(())` if the participation flag updates were successful.
///
pub fn process_participation_flag_updates<E: EthSpec>(
    state: &mut BeaconState<E>,
) -> Result<(), EpochProcessingError> {
    // Update previous epoch participation with current epoch's participation.
    *state.previous_epoch_participation_mut()? =
        std::mem::take(state.current_epoch_participation_mut()?);

    // Initialize current epoch participation with default flags for all validators.
    *state.current_epoch_participation_mut()? =
        List::repeat(ParticipationFlags::default(), state.validators().len())?;

    Ok(())
}
