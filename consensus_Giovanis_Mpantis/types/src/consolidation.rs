use crate::test_utils::TestRandom;
use crate::Epoch;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Struct representing a consolidation event.
///
/// This struct captures details about a consolidation event, including the source index,
/// target index, and the epoch when the consolidation occurred.
#[derive(
    arbitrary::Arbitrary,  // Implements arbitrary generation for testing.
    Debug,                  // Enables debug printing.
    PartialEq,              // Implements PartialEq for comparison.
    Eq,                     // Implements Eq for equality comparison.
    Hash,                   // Implements Hash for hashing capabilities.
    Clone,                  // Implements Clone for creating copies.
    Serialize,              // Implements Serialize for serialization.
    Deserialize,            // Implements Deserialize for deserialization.
    Encode,                 // Implements Encode for SSZ encoding.
    Decode,                 // Implements Decode for SSZ decoding.
    TreeHash,               // Implements TreeHash for Merkle tree hashing.
    TestRandom              // Implements TestRandom for property-based testing.
)]
pub struct Consolidation {
    #[serde(with = "serde_utils::quoted_u64")]
    pub source_index: u64,   // Source index of the consolidation event.
    #[serde(with = "serde_utils::quoted_u64")]
    pub target_index: u64,   // Target index of the consolidation event.
    pub epoch: Epoch,        // Epoch when the consolidation event occurred.
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implements SSZ and TreeHash tests for the Consolidation struct.
    ssz_and_tree_hash_tests!(Consolidation);
}
