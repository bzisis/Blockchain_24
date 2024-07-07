use types::*;

/// Returns validator indices which participated in the attestation, sorted by increasing index.
///
/// # Arguments
///
/// * `committee` - List of validator indices in the committee for the attestation.
/// * `bitlist` - Bitlist indicating which validators in the `committee` participated.
///
/// # Errors
///
/// Returns `Err` if the length of `bitlist` does not match the length of `committee`.
///
/// # Returns
///
/// Returns a `Vec<u64>` containing validator indices sorted in increasing order,
/// representing validators that participated in the attestation.
pub fn get_attesting_indices<E: EthSpec>(
    committee: &[usize],
    bitlist: &BitList<E::MaxValidatorsPerCommittee>,
) -> Result<Vec<u64>, BeaconStateError> {
    if bitlist.len() != committee.len() {
        return Err(BeaconStateError::InvalidBitfield);
    }

    let mut indices = Vec::with_capacity(bitlist.num_set_bits());

    for (i, validator_index) in committee.iter().enumerate() {
        if let Ok(true) = bitlist.get(i) {
            indices.push(*validator_index as u64)
        }
    }

    indices.sort_unstable();

    Ok(indices)
}

/// Shortcut for getting the attesting indices while fetching the committee from the state's cache.
///
/// # Arguments
///
/// * `state` - BeaconState object containing the current state of the beacon chain.
/// * `att` - Attestation object for which to determine the attesting validator indices.
///
/// # Errors
///
/// Returns `Err` if fetching the committee from `state` fails or if `get_attesting_indices` returns an error.
///
/// # Returns
///
/// Returns a `Vec<u64>` containing validator indices sorted in increasing order,
/// representing validators that participated in the attestation.
pub fn get_attesting_indices_from_state<E: EthSpec>(
    state: &BeaconState<E>,
    att: &Attestation<E>,
) -> Result<Vec<u64>, BeaconStateError> {
    let committee = state.get_beacon_committee(att.data.slot, att.data.index)?;
    get_attesting_indices::<E>(committee.committee, &att.aggregation_bits)
}
