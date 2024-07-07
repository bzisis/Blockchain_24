use crate::per_epoch_processing::base::TotalBalances;
use crate::per_epoch_processing::Error;
use crate::per_epoch_processing::{
    weigh_justification_and_finalization, JustificationAndFinalizationState,
};
use safe_arith::SafeArith;
use types::{BeaconState, ChainSpec, EthSpec};

/// Update the justified and finalized checkpoints based on matching target attestations.
///
/// This function processes justification and finalization for a given BeaconState based on the
/// total balances of validators.
///
/// # Arguments
///
/// * `state` - The BeaconState for which to update justification and finalization.
/// * `total_balances` - Total balances of validators for current and previous epochs.
/// * `_spec` - Chain specification defining protocol parameters (currently unused).
///
/// # Returns
///
/// Returns a `JustificationAndFinalizationState` containing updated justification and
/// finalization checkpoints, or an `Error` if processing fails.
///
pub fn process_justification_and_finalization<E: EthSpec>(
    state: &BeaconState<E>,
    total_balances: &TotalBalances,
    _spec: &ChainSpec,
) -> Result<JustificationAndFinalizationState<E>, Error> {
    let justification_and_finalization_state = JustificationAndFinalizationState::new(state);

    // Early return if the current epoch is before or at the genesis epoch plus one.
    if state.current_epoch() <= E::genesis_epoch().safe_add(1)? {
        return Ok(justification_and_finalization_state);
    }

    // Perform the weight calculation for justification and finalization.
    weigh_justification_and_finalization(
        justification_and_finalization_state,
        total_balances.current_epoch(),
        total_balances.previous_epoch_target_attesters(),
        total_balances.current_epoch_target_attesters(),
    )
}
