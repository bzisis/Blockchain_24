use safe_arith::SafeArith;
use std::cmp::max;
use types::{BeaconStateError as Error, *};

/// Initiate the exit of the validator at the specified `index`.
///
/// This function initiates the exit of a validator by setting its exit epoch and withdrawable
/// epoch based on the current state and specifications.
///
/// # Arguments
///
/// * `state` - Mutable reference to the BeaconState where the validator is being exited.
/// * `index` - Index of the validator in the BeaconState validator list.
/// * `spec` - Chain specifications defining validator behavior.
///
/// # Errors
///
/// Returns an `Error` if there is an issue during the exit initiation process.
///
/// # Implementation Details
///
/// This function prepares the exit cache, computes necessary epochs, and updates the validator's
/// exit status accordingly. It optimizes by calculating values before expensive operations to
/// minimize redundant computations.
pub fn initiate_validator_exit<E: EthSpec>(
    state: &mut BeaconState<E>,
    index: usize,
    spec: &ChainSpec,
) -> Result<(), Error> {
    // Ensure the exit cache is built.
    state.build_exit_cache(spec)?;

    // Compute exit queue epoch
    let delayed_epoch = state.compute_activation_exit_epoch(state.current_epoch(), spec)?;
    let mut exit_queue_epoch = state
        .exit_cache()
        .max_epoch()?
        .map_or(delayed_epoch, |epoch| max(epoch, delayed_epoch));
    let exit_queue_churn = state.exit_cache().get_churn_at(exit_queue_epoch)?;

    // Adjust exit queue epoch if churn exceeds the churn limit
    if exit_queue_churn >= state.get_validator_churn_limit(spec)? {
        exit_queue_epoch.safe_add_assign(1)?;
    }

    // Retrieve validator from state
    let validator = state.get_validator_cow(index)?;

    // Return early if the validator has already initiated exit
    if validator.exit_epoch != spec.far_future_epoch {
        return Ok(());
    }

    // Mutate validator's exit and withdrawable epochs
    let mut validator = validator.into_mut()?;
    validator.exit_epoch = exit_queue_epoch;
    validator.withdrawable_epoch =
        exit_queue_epoch.safe_add(spec.min_validator_withdrawability_delay)?;

    // Record validator exit in the exit cache
    state.exit_cache_mut().record_validator_exit(exit_queue_epoch)?;

    Ok(())
}
