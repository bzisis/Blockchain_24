use super::{AggregateSignature, EthSpec, SignedRoot};
use crate::slot_data::SlotData;
use crate::{test_utils::TestRandom, BitVector, Hash256, Slot, SyncCommitteeMessage};
use safe_arith::ArithError;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Errors that can occur when dealing with `SyncCommitteeContribution`.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Error originating from `ssz_types`.
    SszTypesError(ssz_types::Error),
    /// Attempted to sign an already signed slot.
    AlreadySigned(usize),
    /// The number of subnets is zero, causing an arithmetic error.
    SubnetCountIsZero(ArithError),
}

/// An aggregation of `SyncCommitteeMessage`s, used in creating a `SignedContributionAndProof`.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    arbitrary::Arbitrary,
)]
#[serde(bound = "E: EthSpec")]
#[arbitrary(bound = "E: EthSpec")]
pub struct SyncCommitteeContribution<E: EthSpec> {
    /// Slot number for this contribution.
    pub slot: Slot,
    /// Root of the beacon block for this contribution.
    pub beacon_block_root: Hash256,
    /// Index of the subcommittee within the broader sync committee.
    #[serde(with = "serde_utils::quoted_u64")]
    pub subcommittee_index: u64,
    /// Bit vector representing which validators signed this contribution.
    pub aggregation_bits: BitVector<E::SyncSubcommitteeSize>,
    /// Aggregate signature over all included `SyncCommitteeMessage` signatures.
    pub signature: AggregateSignature,
}

impl<E: EthSpec> SyncCommitteeContribution<E> {
    /// Create a `SyncCommitteeContribution` from a `SyncCommitteeMessage`.
    ///
    /// # Arguments
    ///
    /// * `message` - A single `SyncCommitteeMessage`.
    /// * `subcommittee_index` - Index of the subcommittee this contribution pertains to.
    /// * `validator_sync_committee_index` - Index of the validator within the subcommittee.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the validator index fails.
    pub fn from_message(
        message: &SyncCommitteeMessage,
        subcommittee_index: u64,
        validator_sync_committee_index: usize,
    ) -> Result<Self, Error> {
        let mut bits = BitVector::new();
        bits.set(validator_sync_committee_index, true)
            .map_err(Error::SszTypesError)?;
        Ok(Self {
            slot: message.slot,
            beacon_block_root: message.beacon_block_root,
            subcommittee_index,
            aggregation_bits: bits,
            signature: AggregateSignature::from(&message.signature),
        })
    }

    /// Check if the aggregation bitfields of this contribution are disjoint from another.
    pub fn signers_disjoint_from(&self, other: &Self) -> bool {
        self.aggregation_bits
            .intersection(&other.aggregation_bits)
            .is_zero()
    }

    /// Aggregate another `SyncCommitteeContribution` into this one.
    ///
    /// # Panics
    ///
    /// Panics if the data fields do not match or if the aggregation bitfields are not disjoint.
    pub fn aggregate(&mut self, other: &Self) {
        debug_assert_eq!(self.slot, other.slot);
        debug_assert_eq!(self.beacon_block_root, other.beacon_block_root);
        debug_assert_eq!(self.subcommittee_index, other.subcommittee_index);
        debug_assert!(self.signers_disjoint_from(other));

        self.aggregation_bits = self.aggregation_bits.union(&other.aggregation_bits);
        self.signature.add_assign_aggregate(&other.signature);
    }
}

impl SignedRoot for Hash256 {}

/// Data structure representing essential information from a `SyncCommitteeContribution`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash, TestRandom)]
pub struct SyncContributionData {
    /// Slot number for this contribution.
    pub slot: Slot,
    /// Root of the beacon block for this contribution.
    pub beacon_block_root: Hash256,
    /// Index of the subcommittee within the broader sync committee.
    pub subcommittee_index: u64,
}

impl SyncContributionData {
    /// Create `SyncContributionData` from a `SyncCommitteeContribution`.
    pub fn from_contribution<E: EthSpec>(signing_data: &SyncCommitteeContribution<E>) -> Self {
        Self {
            slot: signing_data.slot,
            beacon_block_root: signing_data.beacon_block_root,
            subcommittee_index: signing_data.subcommittee_index,
        }
    }
}

impl<E: EthSpec> SlotData for SyncCommitteeContribution<E> {
    fn get_slot(&self) -> Slot {
        self.slot
    }
}

impl SlotData for SyncContributionData {
    fn get_slot(&self) -> Slot {
        self.slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    ssz_and_tree_hash_tests!(SyncCommitteeContribution<MainnetEthSpec>);
}
