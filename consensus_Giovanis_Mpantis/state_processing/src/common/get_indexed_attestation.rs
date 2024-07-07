use super::get_attesting_indices;
use crate::per_block_processing::errors::{AttestationInvalid as Invalid, BlockOperationError};
use types::*;

type Result<T> = std::result::Result<T, BlockOperationError<Invalid>>;

/// Convert `attestation` to (almost) indexed-verifiable form.
///
/// Spec v0.12.1
pub fn get_indexed_attestation<E: EthSpec>(
    committee: &[usize],
    attestation: &Attestation<E>,
) -> Result<IndexedAttestation<E>> {
    let attesting_indices = get_attesting_indices::<E>(committee, &attestation.aggregation_bits)?;

    Ok(IndexedAttestation {use super::get_attesting_indices;
        use crate::per_block_processing::errors::{AttestationInvalid as Invalid, BlockOperationError};
        use types::*;
        
        type Result<T> = std::result::Result<T, BlockOperationError<Invalid>>;
        
        /// Convert `attestation` to (almost) indexed-verifiable form.
        ///
        /// This function takes a `committee` of validator indices and an `attestation`, and returns
        /// an `IndexedAttestation` which is almost in indexed-verifiable form. It retrieves the attesting
        /// indices using `get_attesting_indices`, and constructs the `IndexedAttestation` struct.
        ///
        /// # Arguments
        ///
        /// * `committee` - A slice of validator indices representing the committee.
        /// * `attestation` - A reference to the attestation to convert.
        ///
        /// # Returns
        ///
        /// A `Result` containing the `IndexedAttestation` if conversion succeeds, or a `BlockOperationError`
        /// with details on why the conversion failed.
        ///
        /// # Errors
        ///
        /// Returns an error if `get_attesting_indices` fails or if creating `VariableList` fails during
        /// construction of `IndexedAttestation`.
        ///
        /// # Spec Compliance
        ///
        /// This function complies with Spec v0.12.1 of the Ethereum 2.0 specifications.
        ///
        pub fn get_indexed_attestation<E: EthSpec>(
            committee: &[usize],
            attestation: &Attestation<E>,
        ) -> Result<IndexedAttestation<E>> {
            let attesting_indices = get_attesting_indices::<E>(committee, &attestation.aggregation_bits)?;
        
            Ok(IndexedAttestation {
                attesting_indices: VariableList::new(attesting_indices)?,
                data: attestation.data.clone(),
                signature: attestation.signature.clone(),
            })
        }
        
        attesting_indices: VariableList::new(attesting_indices)?,
        data: attestation.data.clone(),
        signature: attestation.signature.clone(),
    })
}
