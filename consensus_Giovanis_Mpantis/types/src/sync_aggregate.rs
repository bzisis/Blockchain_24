use crate::consts::altair::SYNC_COMMITTEE_SUBNET_COUNT;
use crate::test_utils::TestRandom;
use crate::{AggregateSignature, BitVector, EthSpec, SyncCommitteeContribution};
use derivative::Derivative;
use safe_arith::{ArithError, SafeArith};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Errors that can occur in operations involving `SyncAggregate`.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Wrapper for errors originating from `ssz_types`.
    SszTypesError(ssz_types::Error),
    /// Wrapper for arithmetic errors from `safe_arith`.
    ArithError(ArithError),
}

impl From<ArithError> for Error {
    fn from(e: ArithError) -> Error {
        Error::ArithError(e)
    }
}

/// SyncAggregate represents an aggregated signature and corresponding bits indicating
/// participation in a sync committee.
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    Derivative,
    arbitrary::Arbitrary,
)]
#[derivative(PartialEq, Hash(bound = "E: EthSpec"))]
#[serde(bound = "E: EthSpec")]
#[arbitrary(bound = "E: EthSpec")]
pub struct SyncAggregate<E: EthSpec> {
    /// Bits indicating which participants are included in the sync committee.
    pub sync_committee_bits: BitVector<E::SyncCommitteeSize>,
    /// Aggregate signature across all participants in the sync committee.
    pub sync_committee_signature: AggregateSignature,
}

impl<E: EthSpec> SyncAggregate<E> {
    /// Creates a new SyncAggregate with default values.
    ///
    /// This method initializes an empty `BitVector` and sets the signature to infinity.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            sync_committee_bits: BitVector::default(),
            sync_committee_signature: AggregateSignature::infinity(),
        }
    }

    /// Creates a `SyncAggregate` from a slice of `SyncCommitteeContribution`s.
    ///
    /// This method aggregates contributions from multiple sync committee contributions,
    /// updating both the `sync_committee_bits` and `sync_committee_signature`.
    ///
    /// Equivalent to `process_sync_committee_contributions` from the spec.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are errors during arithmetic operations or when
    /// setting bits in the `sync_committee_bits`.
    pub fn from_contributions(
        contributions: &[SyncCommitteeContribution<E>],
    ) -> Result<SyncAggregate<E>, Error> {
        let mut sync_aggregate = Self::new();
        let sync_subcommittee_size =
            E::sync_committee_size().safe_div(SYNC_COMMITTEE_SUBNET_COUNT as usize)?;
        for contribution in contributions {
            for (index, participated) in contribution.aggregation_bits.iter().enumerate() {
                if *participated {
                    let participant_index = sync_subcommittee_size
                        .safe_mul(contribution.subcommittee_index as usize)?
                        .safe_add(index)?;
                    sync_aggregate
                        .sync_committee_bits
                        .set(participant_index, true)
                        .map_err(Error::SszTypesError)?;
                }
            }
            sync_aggregate
                .sync_committee_signature
                .add_assign_aggregate(&contribution.signature);
        }
        Ok(sync_aggregate)
    }

    /// Creates an empty `SyncAggregate` to be used at genesis.
    ///
    /// This method initializes an empty `BitVector` and sets the signature to empty.
    ///
    /// This should not be used as the starting point for aggregation; use `new` instead.
    pub fn empty() -> Self {
        Self {
            sync_committee_bits: BitVector::default(),
            sync_committee_signature: AggregateSignature::empty(),
        }
    }

    /// Returns the number of bits that are `true` in `self.sync_committee_bits`.
    ///
    /// This method calculates the count of set bits in the `BitVector`.
    pub fn num_set_bits(&self) -> usize {
        self.sync_committee_bits.num_set_bits()
    }
}
