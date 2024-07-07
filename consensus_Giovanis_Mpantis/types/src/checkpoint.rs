use crate::test_utils::TestRandom;
use crate::{Epoch, Hash256};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Casper FFG checkpoint, used in attestations.
///
/// Represents a checkpoint in the Casper FFG protocol, used primarily in attestations.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct Checkpoint {
    /// Epoch number corresponding to the checkpoint.
    pub epoch: Epoch,
    /// Root hash representing the checkpoint.
    pub root: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ and tree hash encoding/decoding consistency for Checkpoint.
    ssz_and_tree_hash_tests!(Checkpoint);
}
