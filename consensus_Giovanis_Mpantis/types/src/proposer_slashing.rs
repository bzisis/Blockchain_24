use crate::test_utils::TestRandom;
use crate::SignedBeaconBlockHeader;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents two conflicting proposals from the same proposer (validator) in the context of a blockchain.
///
/// This struct is defined according to the specification v0.12.1.
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
pub struct ProposerSlashing {
    /// The first signed header proposing a block.
    pub signed_header_1: SignedBeaconBlockHeader,
    /// The second signed header proposing a conflicting block.
    pub signed_header_2: SignedBeaconBlockHeader,
}

impl ProposerSlashing {
    /// Returns the proposer index associated with the slashing event.
    ///
    /// This method assumes that the validity of the slashing has already been checked.
    pub fn proposer_index(&self) -> u64 {
        self.signed_header_1.message.proposer_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ serialization and tree hashing tests are implemented for ProposerSlashing.
    ssz_and_tree_hash_tests!(ProposerSlashing);
}
