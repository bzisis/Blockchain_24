/// Increase the balance of a validator in the BeaconState.
///
/// # Arguments
///
/// * `state` - The mutable reference to the BeaconState.
/// * `index` - The index of the validator whose balance should be increased.
/// * `delta` - The amount by which to increase the balance.
///
/// # Errors
///
/// Returns an error if the balance cannot be increased due to overflow.
///
/// # Examples
///
/// ```
/// use types::{BeaconState, EthSpec};
/// use your_module::increase_balance;
///
/// let mut state: BeaconState<EthSpec> = BeaconState::default();
/// let index = 0;
/// let delta = 100;
///
/// let result = increase_balance(&mut state, index, delta);
/// assert!(result.is_ok());
/// ```
pub fn increase_balance<E: EthSpec>(
    state: &mut BeaconState<E>,
    index: usize,
    delta: u64,
) -> Result<(), BeaconStateError> {
    increase_balance_directly(state.get_balance_mut(index)?, delta)
}

/// Decrease the balance of a validator in the BeaconState.
///
/// # Arguments
///
/// * `state` - The mutable reference to the BeaconState.
/// * `index` - The index of the validator whose balance should be decreased.
/// * `delta` - The amount by which to decrease the balance.
///
/// # Examples
///
/// ```
/// use types::{BeaconState, EthSpec};
/// use your_module::decrease_balance;
///
/// let mut state: BeaconState<EthSpec> = BeaconState::default();
/// let index = 0;
/// let delta = 100;
///
/// decrease_balance(&mut state, index, delta);
/// // Balance of the validator at index 0 is decreased by 100.
/// ```
pub fn decrease_balance<E: EthSpec>(
    state: &mut BeaconState<E>,
    index: usize,
    delta: u64,
) -> Result<(), BeaconStateError> {
    decrease_balance_directly(state.get_balance_mut(index)?, delta)
}
