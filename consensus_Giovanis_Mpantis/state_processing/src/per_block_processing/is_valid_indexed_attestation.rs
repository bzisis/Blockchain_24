use super::errors::{BlockOperationError, IndexedAttestationInvalid as Invalid};
use super::signature_sets::{get_pubkey_from_state, indexed_attestation_signature_set};
use crate::VerifySignatures;
use itertools::Itertools;
use types::*;

/// Alias for the result type used in this function.
type Result<T> = std::result::Result<T, BlockOperationError<Invalid>>;

/// Creates a `BlockOperationError` from an `Invalid` reason.
fn error(reason: Invalid) -> BlockOperationError<Invalid> {
    BlockOperationError::invalid(reason)
}

/// Verify an `IndexedAttestation`.
///
/// This function checks the validity of an `IndexedAttestation` against the provided
/// `BeaconState`, `VerifySignatures` setting, and `ChainSpec`.
///
/// # Arguments
///
/// * `state` - The current BeaconState.
/// * `indexed_attestation` - The `IndexedAttestation` to be verified.
/// * `verify_signatures` - Specifies whether to verify signatures.
/// * `spec` - The `ChainSpec` defining the blockchain parameters.
///
/// # Returns
///
/// Returns `Ok(())` if the attestation is valid, otherwise returns an error of type
/// `BlockOperationError<Invalid>`.
pub fn is_valid_indexed_attestation<E: EthSpec>(
    state: &BeaconState<E>,
    indexed_attestation: &IndexedAttestation<E>,
    verify_signatures: VerifySignatures,
    spec: &ChainSpec,
) -> Result<()> {
    let indices = &indexed_attestation.attesting_indices;

    // Verify that indices aren't empty
    verify!(!indices.is_empty(), Invalid::IndicesEmpty);

    // Check that indices are sorted and unique
    let check_sorted = |list: &[u64]| -> Result<()> {
        list.iter()
            .tuple_windows()
            .enumerate()
            .try_for_each(|(i, (x, y))| {
                if x < y {
                    Ok(())
                } else {
                    Err(error(Invalid::BadValidatorIndicesOrdering(i)))
                }
            })?;
        Ok(())
    };
    check_sorted(indices)?;

    // Verify signatures if requested
    if verify_signatures.is_true() {
        verify!(
            indexed_attestation_signature_set(
                state,
                |i| get_pubkey_from_state(state, i),
                &indexed_attestation.signature,
                indexed_attestation,
                spec
            )?
            .verify(),
            Invalid::BadSignature
        );
    }

    Ok(())
}
