use crate::per_epoch_processing::single_pass::{process_epoch_single_pass, SinglePassConfig};
use crate::EpochProcessingError;
use types::beacon_state::BeaconState;
use types::chain_spec::ChainSpec;
use types::eth_spec::EthSpec;

/// Slow version of `process_inactivity_updates` that runs a subset of single-pass processing.
///
/// Should not be used for block processing, but is useful for testing & analytics.
///
/// This function processes inactivity updates for the given `state` using a slower method
/// that disables certain optimizations. It should primarily be used for testing or
/// analytical purposes, and not for actual block processing due to its slower execution.
///
/// # Arguments
///
/// * `state` - The mutable reference to the beacon state to be processed.
/// * `spec` - The chain specification that defines the rules and parameters for processing.
///
/// # Errors
///
/// Returns an error of type `EpochProcessingError` if any error occurs during processing.
///
/// # Examples
///
/// ```rust
/// use crate::per_epoch_processing::process_inactivity_updates_slow;
/// use types::{BeaconState, ChainSpec, MainnetEthSpec};
///
/// fn main() {
///     let mut state: BeaconState<MainnetEthSpec> = BeaconState::default();
///     let spec: ChainSpec = ChainSpec::mainnet();
///
///     // Process inactivity updates with the slow method
///     let result = process_inactivity_updates_slow(&mut state, &spec);
///     assert!(result.is_ok());
/// }
/// ```
pub fn process_inactivity_updates_slow<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<(), EpochProcessingError> {
    process_epoch_single_pass(
        state,
        spec,
        SinglePassConfig {
            inactivity_updates: true,
            ..SinglePassConfig::disable_all()
        },
    )?;
    Ok(())
}
