use derivative::Derivative;
use safe_arith::ArithError;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

use crate::slot_data::SlotData;
use crate::{test_utils::TestRandom, Hash256, Slot};

use super::{
    AggregateSignature, AttestationData, BitList, ChainSpec, Domain, EthSpec, Fork, SecretKey,
    Signature, SignedRoot,
};

/// Errors that can occur when working with attestations.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// An error from `ssz_types` crate.
    SszTypesError(ssz_types::Error),
    /// The specified validator has already signed this attestation.
    AlreadySigned(usize),
    /// The subnet count is zero, causing an arithmetic error.
    SubnetCountIsZero(ArithError),
}

/// Details an attestation that can be slashable.
///
/// This struct represents an attestation, which details the agreement of validators on some data.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    Derivative,
)]
#[derivative(PartialEq, Hash(bound = "E: EthSpec"))]
#[serde(bound = "E: EthSpec")]
#[arbitrary(bound = "E: EthSpec")]
pub struct Attestation<E: EthSpec> {
    /// Bitlist of validator indices that signed the attestation.
    pub aggregation_bits: BitList<E::MaxValidatorsPerCommittee>,
    /// Attestation data.
    pub data: AttestationData,
    /// Aggregate signature covering the attestation data.
    pub signature: AggregateSignature,
}

impl<E: EthSpec> Attestation<E> {
    /// Check if the aggregation bitfields of two attestations are disjoint.
    ///
    /// This method checks whether the validators who signed this attestation are disjoint
    /// from those who signed another attestation.
    pub fn signers_disjoint_from(&self, other: &Self) -> bool {
        self.aggregation_bits
            .intersection(&other.aggregation_bits)
            .is_zero()
    }

    /// Aggregate another `Attestation` into this one.
    ///
    /// The aggregation bitfields must be disjoint, and the data must be the same.
    ///
    /// # Panics
    ///
    /// Panics if the data of `self` does not match `other`.
    pub fn aggregate(&mut self, other: &Self) {
        debug_assert_eq!(self.data, other.data);
        debug_assert!(self.signers_disjoint_from(other));

        self.aggregation_bits = self.aggregation_bits.union(&other.aggregation_bits);
        self.signature.add_assign_aggregate(&other.signature);
    }

    /// Signs `self`, setting the `committee_position`'th bit of `aggregation_bits` to `true`.
    ///
    /// Returns an `AlreadySigned` error if the `committee_position`'th bit is already `true`.
    pub fn sign(
        &mut self,
        secret_key: &SecretKey,
        committee_position: usize,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> Result<(), Error> {
        let domain = spec.get_domain(
            self.data.target.epoch,
            Domain::BeaconAttester,
            fork,
            genesis_validators_root,
        );
        let message = self.data.signing_root(domain);

        self.add_signature(&secret_key.sign(message), committee_position)
    }

    /// Adds `signature` to `self` and sets the `committee_position`'th bit of `aggregation_bits` to `true`.
    ///
    /// Returns an `AlreadySigned` error if the `committee_position`'th bit is already `true`.
    pub fn add_signature(
        &mut self,
        signature: &Signature,
        committee_position: usize,
    ) -> Result<(), Error> {
        if self
            .aggregation_bits
            .get(committee_position)
            .map_err(Error::SszTypesError)?
        {
            Err(Error::AlreadySigned(committee_position))
        } else {
            self.aggregation_bits
                .set(committee_position, true)
                .map_err(Error::SszTypesError)?;

            self.signature.add_assign(signature);

            Ok(())
        }
    }
}

impl<E: EthSpec> SlotData for Attestation<E> {
    /// Returns the slot associated with this attestation data.
    fn get_slot(&self) -> Slot {
        self.data.slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    // Check the in-memory size of an `Attestation`, which is useful for reasoning about memory
    // and preventing regressions.
    //
    // This test will only pass with `blst`, if we run these tests with another
    // BLS library in future we will have to make it generic.
    #[test]
    fn size_of() {
        use std::mem::size_of;

        let aggregation_bits =
            size_of::<BitList<<MainnetEthSpec as EthSpec>::MaxValidatorsPerCommittee>>();
        let attestation_data = size_of::<AttestationData>();
        let signature = size_of::<AggregateSignature>();

        assert_eq!(aggregation_bits, 56);
        assert_eq!(attestation_data, 128);
        assert_eq!(signature, 288 + 16);

        let attestation_expected = aggregation_bits + attestation_data + signature;
        assert_eq!(attestation_expected, 488);
        assert_eq!(
            size_of::<Attestation<MainnetEthSpec>>(),
            attestation_expected
        );
    }

    ssz_and_tree_hash_tests!(Attestation<MainnetEthSpec>);
}
