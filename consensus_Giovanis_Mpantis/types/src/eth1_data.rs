use super::Hash256;
use crate::test_utils::TestRandom;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Contains data obtained from the Eth1 chain.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Clone,
    Default,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct Eth1Data {
    /// Root of the deposit tree.
    pub deposit_root: Hash256,

    /// Number of deposits processed.
    #[serde(with = "serde_utils::quoted_u64")]
    pub deposit_count: u64,

    /// Hash of the block this Eth1Data is associated with.
    pub block_hash: Hash256,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Automatically generates SSZ and Merkle tree hash tests for Eth1Data.
    ssz_and_tree_hash_tests!(Eth1Data);
}
