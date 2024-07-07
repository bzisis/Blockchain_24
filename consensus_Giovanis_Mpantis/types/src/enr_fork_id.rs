use crate::test_utils::TestRandom;
use crate::Epoch;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Specifies a fork which allows nodes to identify each other on the network. This fork is used in
/// a node's local ENR.
///
/// Spec v0.11
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct EnrForkId {
    /// The digest of the fork, represented as a 4-byte array.
    #[serde(with = "serde_utils::bytes_4_hex")]
    pub fork_digest: [u8; 4],

    /// The version of the next fork, represented as a 4-byte array.
    #[serde(with = "serde_utils::bytes_4_hex")]
    pub next_fork_version: [u8; 4],

    /// The epoch at which the next fork is expected to occur.
    pub next_fork_epoch: Epoch,
}

#[cfg(test)]
mod tests {
    use super::*;

    // This macro generates SSZ and tree hash tests for EnrForkId.
    ssz_and_tree_hash_tests!(EnrForkId);
}
