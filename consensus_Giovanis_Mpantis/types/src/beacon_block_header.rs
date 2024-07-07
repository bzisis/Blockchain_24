use crate::test_utils::TestRandom;
use crate::*;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

/// A header of a `BeaconBlock`.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct BeaconBlockHeader {
    /// Slot number of the block.
    pub slot: Slot,
    /// Index of the proposer for this block.
    #[serde(with = "serde_utils::quoted_u64")]
    pub proposer_index: u64,
    /// Hash of the parent block's header.
    pub parent_root: Hash256,
    /// Root of the block's state.
    pub state_root: Hash256,
    /// Root of the block's body.
    pub body_root: Hash256,
}

impl SignedRoot for BeaconBlockHeader {}

impl BeaconBlockHeader {
    /// Returns the canonical root of the header.
    ///
    /// This function computes the `tree_hash_root` of the header and returns it.
    ///
    /// Spec v0.12.1
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.tree_hash_root()[..])
    }

    /// Signs the header, producing a `SignedBeaconBlockHeader`.
    ///
    /// This function signs the header using the provided `secret_key`, `fork`, `genesis_validators_root`,
    /// and `spec` parameters and returns a `SignedBeaconBlockHeader`.
    ///
    /// # Arguments
    ///
    /// * `secret_key` - The secret key used for signing.
    /// * `fork` - The fork identifier.
    /// * `genesis_validators_root` - The root hash of genesis validators.
    /// * `spec` - The chain specification.
    ///
    /// # Returns
    ///
    /// A `SignedBeaconBlockHeader` containing the signed header and its signature.
    pub fn sign<E: EthSpec>(
        self,
        secret_key: &SecretKey,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> SignedBeaconBlockHeader {
        let epoch = self.slot.epoch(E::slots_per_epoch());
        let domain = spec.get_domain(epoch, Domain::BeaconProposer, fork, genesis_validators_root);
        let message = self.signing_root(domain);
        let signature = secret_key.sign(message);
        SignedBeaconBlockHeader {
            message: self,
            signature,
        }
    }

    /// Creates an empty `BeaconBlockHeader` with default values.
    ///
    /// This function constructs a new `BeaconBlockHeader` with all fields initialized to their default values.
    ///
    /// # Returns
    ///
    /// An empty `BeaconBlockHeader`.
    pub fn empty() -> Self {
        Self {
            body_root: Default::default(),
            parent_root: Default::default(),
            proposer_index: Default::default(),
            slot: Default::default(),
            state_root: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(BeaconBlockHeader);
}
