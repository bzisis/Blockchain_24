use crate::test_utils::TestRandom;
use crate::{Hash256, SignedRoot};

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Specifies data related to a fork of the `BeaconChain`, used to prevent replay attacks.
///
/// This struct defines information about a fork, including its current version and the root hash
/// of genesis validators for signature verification purposes.
///
/// Spec v0.12.1
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
pub struct ForkData {
    /// Current version identifier of the fork, serialized as a 4-byte array in hexadecimal.
    #[serde(with = "serde_utils::bytes_4_hex")]
    pub current_version: [u8; 4],
    /// Root hash of genesis validators, used for signature verification related to this fork.
    pub genesis_validators_root: Hash256,
}

impl SignedRoot for ForkData {}

#[cfg(test)]
mod tests {
    use super::*;

    // Generate SSZ serialization and tree hashing tests for ForkData.
    ssz_and_tree_hash_tests!(ForkData);
}
