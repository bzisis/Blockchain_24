use crate::common::decrease_balance;
use crate::per_epoch_processing::{
    single_pass::{process_epoch_single_pass, SinglePassConfig},
    Error,
};
use safe_arith::{SafeArith, SafeArithIter};
use types::{BeaconState, ChainSpec, EthSpec, Unsigned};

/// Process slashings for the current epoch.
///
/// Computes penalties for validators who have been slashed and adjusts their balances accordingly.
///
/// # Arguments
///
/// * `state` - The mutable reference to the beacon state.
/// * `total_balance` - The total balance of the validator set.
/// * `spec` - The chain specification defining the rules and parameters of the chain.
///
/// # Errors
///
/// Returns an error if any arithmetic operation fails during the computation of penalties.
///
/// # Examples
///
/// ```rust
/// use types::{BeaconState, ChainSpec, MainnetEthSpec};
/// use beacon_processing::process_slashings;
///
/// let mut state: BeaconState<MainnetEthSpec> = BeaconState::default();
/// let spec: ChainSpec = ChainSpec::default();
/// let total_balance: u64 = 1_000_000;
///
/// assert!(process_slashings(&mut state, total_balance, &spec).is_ok());
/// ```
pub fn process_slashings<E: EthSpec>(
    state: &mut BeaconState<E>,
    total_balance: u64,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let epoch = state.current_epoch();
    let sum_slashings = state.get_all_slashings().iter().copied().safe_sum()?;

    let adjusted_total_slashing_balance = std::cmp::min(
        sum_slashings.safe_mul(spec.proportional_slashing_multiplier_for_state(state))?,
        total_balance,
    );

    let target_withdrawable_epoch =
        epoch.safe_add(E::EpochsPerSlashingsVector::to_u64().safe_div(2)?)?;
    let indices = state
        .validators()
        .iter()
        .enumerate()
        .filter(|(_, validator)| {
            validator.slashed && target_withdrawable_epoch == validator.withdrawable_epoch
        })
        .map(|(index, validator)| (index, validator.effective_balance))
        .collect::<Vec<(usize, u64)>>();

    for (index, validator_effective_balance) in indices {
        let increment = spec.effective_balance_increment;
        let penalty_numerator = validator_effective_balance
            .safe_div(increment)?
            .safe_mul(adjusted_total_slashing_balance)?;
        let penalty = penalty_numerator
            .safe_div(total_balance)?
            .safe_mul(increment)?;

        decrease_balance(state, index, penalty)?;
    }

    Ok(())
}

/// Process slashings for the current epoch in a slow manner.
///
/// This function processes slashings by performing a single pass over the epoch state.
/// It sets up the configuration to enable slashing processing and disables other types of processing.
///
/// # Arguments
///
/// * `state` - The mutable reference to the beacon state.
/// * `spec` - The chain specification defining the rules and parameters of the chain.
///
/// # Errors
///
/// Returns an error if the single-pass processing encounters any issues.
///
/// # Examples
///
/// ```rust
/// use types::{BeaconState, ChainSpec, MainnetEthSpec};
/// use beacon_processing::process_slashings_slow;
///
/// let mut state: BeaconState<MainnetEthSpec> = BeaconState::default();
/// let spec: ChainSpec = ChainSpec::default();
///
/// assert!(process_slashings_slow(&mut state, &spec).is_ok());
/// ```
pub fn process_slashings_slow<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    process_epoch_single_pass(
        state,
        spec,
        SinglePassConfig {
            slashings: true,
            ..SinglePassConfig::disable_all()
        },
    )?;
    Ok(())
}
