use crate::test_utils::TestRandom;
use crate::{EthSpec, FixedVector, SyncSubnetId};
use bls::PublicKeyBytes;
use safe_arith::{ArithError, SafeArith};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use std::collections::HashMap;
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Errors that can occur in the context of `SyncCommittee`.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Arithmetic errors encountered during safe arithmetic operations.
    ArithError(ArithError),
    /// Indicates an invalid range of subcommittee indices.
    InvalidSubcommitteeRange {
        /// Start index of the subcommittee range.
        start_subcommittee_index: usize,
        /// End index of the subcommittee range.
        end_subcommittee_index: usize,
        /// The specific subcommittee index that caused the error.
        subcommittee_index: usize,
    },
}

impl From<ArithError> for Error {
    /// Convert `ArithError` into `Error`.
    fn from(e: ArithError) -> Error {
        Error::ArithError(e)
    }
}

/// Represents a sync committee in Ethereum 2.0, parameterized over `EthSpec`.
#[derive(
    Debug,
    PartialEq,
    Clone,
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
pub struct SyncCommittee<E: EthSpec> {
    /// Public keys of validators in the sync committee.
    pub pubkeys: FixedVector<PublicKeyBytes, E::SyncCommitteeSize>,
    /// Aggregate public key of the sync committee.
    pub aggregate_pubkey: PublicKeyBytes,
}

impl<E: EthSpec> SyncCommittee<E> {
    /// Create a temporary sync committee that should *never* be included in a legitimate consensus object.
    pub fn temporary() -> Self {
        Self {
            pubkeys: FixedVector::from_elem(PublicKeyBytes::empty()),
            aggregate_pubkey: PublicKeyBytes::empty(),
        }
    }

    /// Return the public keys in the sync committee for the given `subcommittee_index`.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the subcommittee index is out of range.
    pub fn get_subcommittee_pubkeys(
        &self,
        subcommittee_index: usize,
    ) -> Result<Vec<PublicKeyBytes>, Error> {
        let start_subcommittee_index = subcommittee_index.safe_mul(E::sync_subcommittee_size())?;
        let end_subcommittee_index =
            start_subcommittee_index.safe_add(E::sync_subcommittee_size())?;
        self.pubkeys
            .get(start_subcommittee_index..end_subcommittee_index)
            .ok_or(Error::InvalidSubcommitteeRange {
                start_subcommittee_index,
                end_subcommittee_index,
                subcommittee_index,
            })
            .map(|s| s.to_vec())
    }

    /// For a given `pubkey`, finds all subcommittees that it is included in, and maps the
    /// subcommittee index (typed as `SyncSubnetId`) to all positions this `pubkey` is associated
    /// with within the subcommittee.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if arithmetic operations fail.
    pub fn subcommittee_positions_for_public_key(
        &self,
        pubkey: &PublicKeyBytes,
    ) -> Result<HashMap<SyncSubnetId, Vec<usize>>, Error> {
        let mut subnet_positions = HashMap::new();
        for (committee_index, validator_pubkey) in self.pubkeys.iter().enumerate() {
            if pubkey == validator_pubkey {
                let subcommittee_index = committee_index.safe_div(E::sync_subcommittee_size())?;
                let position_in_subcommittee =
                    committee_index.safe_rem(E::sync_subcommittee_size())?;
                subnet_positions
                    .entry(SyncSubnetId::new(subcommittee_index as u64))
                    .or_insert_with(Vec::new)
                    .push(position_in_subcommittee);
            }
        }
        Ok(subnet_positions)
    }

    /// Returns `true` if the `pubkey` exists in the `SyncCommittee`.
    pub fn contains(&self, pubkey: &PublicKeyBytes) -> bool {
        self.pubkeys.contains(pubkey)
    }
}
