/// A collection of all functions that mutate the `ProgressiveBalancesCache`.
use crate::metrics::{
    PARTICIPATION_CURR_EPOCH_TARGET_ATTESTING_GWEI_PROGRESSIVE_TOTAL,
    PARTICIPATION_PREV_EPOCH_TARGET_ATTESTING_GWEI_PROGRESSIVE_TOTAL,
};
use crate::{BlockProcessingError, EpochProcessingError};
use lighthouse_metrics::set_gauge;
use types::{
    is_progressive_balances_enabled, BeaconState, BeaconStateError, ChainSpec, Epoch,
    EpochTotalBalances, EthSpec, ParticipationFlags, ProgressiveBalancesCache, Validator,
};

/// Initializes the `ProgressiveBalancesCache` if it is unbuilt.
///
/// This function checks if progressive balances are enabled and if the cache is already
/// initialized. If not, it calculates the total flag balances for the previous and current
/// epochs, excluding slashed validators, and initializes the cache with these values.
///
/// # Arguments
///
/// * `state` - Mutable reference to the `BeaconState`.
/// * `spec` - Reference to the `ChainSpec`.
///
/// # Returns
///
/// Returns `Ok(())` if the cache is successfully initialized or already initialized, otherwise
/// returns an error.
pub fn initialize_progressive_balances_cache<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), BeaconStateError> {
    if !is_progressive_balances_enabled(state)
        || state.progressive_balances_cache().is_initialized()
    {
        return Ok(());
    }

    // Calculate the total flag balances for previous & current epoch in a single iteration.
    // This calculates `get_total_balance(unslashed_participating_indices(..))` for each flag in
    // the current and previous epoch.
    let current_epoch = state.current_epoch();
    let previous_epoch = state.previous_epoch();
    let mut previous_epoch_cache = EpochTotalBalances::new(spec);
    let mut current_epoch_cache = EpochTotalBalances::new(spec);
    for ((validator, current_epoch_flags), previous_epoch_flags) in state
        .validators()
        .iter()
        .zip(state.current_epoch_participation()?)
        .zip(state.previous_epoch_participation()?)
    {
        // Exclude slashed validators. We are calculating *unslashed* participating totals.
        if validator.slashed {
            continue;
        }

        // Update current epoch flag balances.
        if validator.is_active_at(current_epoch) {
            update_flag_total_balances(&mut current_epoch_cache, *current_epoch_flags, validator)?;
        }
        // Update previous epoch flag balances.
        if validator.is_active_at(previous_epoch) {
            update_flag_total_balances(
                &mut previous_epoch_cache,
                *previous_epoch_flags,
                validator,
            )?;
        }
    }

    state.progressive_balances_cache_mut().initialize(
        current_epoch,
        previous_epoch_cache,
        current_epoch_cache,
    );

    update_progressive_balances_metrics(state.progressive_balances_cache())?;

    Ok(())
}

/// Updates the total balances for each participation flag in an epoch.
///
/// During the initialization of the progressive balances for a single epoch, this function adds
/// `validator.effective_balance` to the flag total, for each flag present in `participation_flags`.
///
/// # Arguments
///
/// * `total_balances` - Mutable reference to the `EpochTotalBalances`.
/// * `participation_flags` - The participation flags for the validator.
/// * `validator` - Reference to the `Validator`.
///
/// # Returns
///
/// Returns `Ok(())` if the balances are successfully updated, otherwise returns an error.
///
/// # Preconditions
///
/// * `validator` must not be slashed.
/// * The `participation_flags` must be for `validator` in the same epoch as the `total_balances`.
fn update_flag_total_balances(
    total_balances: &mut EpochTotalBalances,
    participation_flags: ParticipationFlags,
    validator: &Validator,
) -> Result<(), BeaconStateError> {
    for (flag, balance) in total_balances.total_flag_balances.iter_mut().enumerate() {
        if participation_flags.has_flag(flag)? {
            balance.safe_add_assign(validator.effective_balance)?;
        }
    }
    Ok(())
}

/// Updates the `ProgressiveBalancesCache` when a new target attestation has been processed.
///
/// This function updates the progressive balances cache with the validator's effective balance
/// when a new attestation is processed.
///
/// # Arguments
///
/// * `state` - Mutable reference to the `BeaconState`.
/// * `epoch` - The epoch of the attestation.
/// * `flag_index` - The index of the participation flag.
/// * `validator_effective_balance` - The effective balance of the validator.
/// * `validator_slashed` - Indicates if the validator is slashed.
///
/// # Returns
///
/// Returns `Ok(())` if the cache is successfully updated, otherwise returns an error.
pub fn update_progressive_balances_on_attestation<E: EthSpec>(
    state: &mut BeaconState<E>,
    epoch: Epoch,
    flag_index: usize,
    validator_effective_balance: u64,
    validator_slashed: bool,
) -> Result<(), BlockProcessingError> {
    if is_progressive_balances_enabled(state) {
        state.progressive_balances_cache_mut().on_new_attestation(
            epoch,
            validator_slashed,
            flag_index,
            validator_effective_balance,
        )?;
    }
    Ok(())
}

/// Updates the `ProgressiveBalancesCache` when a target attester has been slashed.
///
/// This function updates the progressive balances cache with the validator's effective balance
/// when a target attester is slashed.
///
/// # Arguments
///
/// * `state` - Mutable reference to the `BeaconState`.
/// * `validator_index` - The index of the slashed validator.
/// * `validator_effective_balance` - The effective balance of the validator.
///
/// # Returns
///
/// Returns `Ok(())` if the cache is successfully updated, otherwise returns an error.
pub fn update_progressive_balances_on_slashing<E: EthSpec>(
    state: &mut BeaconState<E>,
    validator_index: usize,
    validator_effective_balance: u64,
) -> Result<(), BlockProcessingError> {
    if is_progressive_balances_enabled(state) {
        let previous_epoch_participation = *state
            .previous_epoch_participation()?
            .get(validator_index)
            .ok_or(BeaconStateError::UnknownValidator(validator_index))?;

        let current_epoch_participation = *state
            .current_epoch_participation()?
            .get(validator_index)
            .ok_or(BeaconStateError::UnknownValidator(validator_index))?;

        state.progressive_balances_cache_mut().on_slashing(
            previous_epoch_participation,
            current_epoch_participation,
            validator_effective_balance,
        )?;
    }

    Ok(())
}

/// Updates the `ProgressiveBalancesCache` on epoch transition.
///
/// This function updates the progressive balances cache during an epoch transition, recalculating
/// the balances as required by the new epoch.
///
/// # Arguments
///
/// * `state` - Mutable reference to the `BeaconState`.
/// * `spec` - Reference to the `ChainSpec`.
///
/// # Returns
///
/// Returns `Ok(())` if the cache is successfully updated, otherwise returns an error.
pub fn update_progressive_balances_on_epoch_transition<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), EpochProcessingError> {
    if is_progressive_balances_enabled(state) {
        state
            .progressive_balances_cache_mut()
            .on_epoch_transition(spec)?;

        update_progressive_balances_metrics(state.progressive_balances_cache())?;
    }

    Ok(())
}

/// Updates the metrics for the progressive balances.
///
/// This function sets the metrics gauges for the previous and current epoch target attesting
/// balances using the values from the `ProgressiveBalancesCache`.
///
/// # Arguments
///
/// * `cache` - Reference to the `ProgressiveBalancesCache`.
///
/// # Returns
///
/// Returns `Ok(())` if the metrics are successfully updated, otherwise returns an error.
pub fn update_progressive_balances_metrics(
    cache: &ProgressiveBalancesCache,
) -> Result<(), BeaconStateError> {
    set_gauge(
        &PARTICIPATION_PREV_EPOCH_TARGET_ATTESTING_GWEI_PROGRESSIVE_TOTAL,
        cache.previous_epoch_target_attesting_balance()? as i64,
    );

    set_gauge(
        &PARTICIPATION_CURR_EPOCH_TARGET_ATTESTING_GWEI_PROGRESSIVE_TOTAL,
        cache.current_epoch_target_attesting_balance()? as i64,
    );

    Ok(())
}
