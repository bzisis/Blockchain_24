use crate::test_utils::TestRandom;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a pending consolidation operation with source and target indices.
///
/// This struct is used to represent pending consolidation operations, where funds or other
/// resources are consolidated from a source index to a target index.
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
pub struct PendingConsolidation {
    /// Source index from which the consolidation operation originates.
    #[serde(with = "serde_utils::quoted_u64")]
    pub source_index: u64,
    /// Target index to which the consolidation operation is directed.
    #[serde(with = "serde_utils::quoted_u64")]
    pub target_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ and TreeHash implementations are correct for PendingConsolidation.
    ssz_and_tree_hash_tests!(PendingConsolidation);
}
