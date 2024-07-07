use crate::per_epoch_processing::{
    single_pass::{process_epoch_single_pass, SinglePassConfig},
    Error,
};
use types::consts::altair::PARTICIPATION_FLAG_WEIGHTS;
use types::{BeaconState, ChainSpec, EthSpec};

/// Apply attester and proposer rewards.
///
/// This function should only be used for testing purposes.
///
/// # Arguments
///
/// * `state` - Mutable reference to the beacon state.
/// * `spec` - Chain specification parameters.
///
/// # Errors
///
/// Returns an error if there was an issue processing rewards and penalties.
///
/// # Example
///
/// ```
/// use types::{BeaconState, ChainSpec, EthSpec};
/// use your_module::process_rewards_and_penalties_slow;
///
/// let mut state: BeaconState<EthSpec> = BeaconState::default();
/// let spec: ChainSpec = ChainSpec::mainnet();
///
/// let result = process_rewards_and_penalties_slow(&mut state, &spec);
/// assert!(result.is_ok());
/// ```
pub fn process_rewards_and_penalties_slow<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), Error> {
    process_epoch_single_pass(
        state,
        spec,
        SinglePassConfig {
            rewards_and_penalties: true,
            ..SinglePassConfig::disable_all()
        },
    )?;
    Ok(())
}

/// Get the weight for a `flag_index` from the constant list of all weights.
///
/// # Arguments
///
/// * `flag_index` - Index of the flag for which weight is requested.
///
/// # Errors
///
/// Returns an error if `flag_index` is out of bounds.
///
/// # Example
///
/// ```
/// use your_module::get_flag_weight;
///
/// let flag_index = 0;
/// let weight_result = get_flag_weight(flag_index);
///
/// assert!(weight_result.is_ok());
/// ```
pub fn get_flag_weight(flag_index: usize) -> Result<u64, Error> {
    PARTICIPATION_FLAG_WEIGHTS
        .get(flag_index)
        .copied()
        .ok_or(Error::InvalidFlagIndex(flag_index))
}
