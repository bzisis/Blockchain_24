use crate::test_utils::TestRandom;
use crate::{SignedRoot, Slot};

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Struct representing data for sync aggregator selection.
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct SyncAggregatorSelectionData {
    /// Slot for which the selection data is applicable.
    pub slot: Slot,
    /// Index of the subcommittee for this selection data.
    #[serde(with = "serde_utils::quoted_u64")]
    pub subcommittee_index: u64,
}

impl SignedRoot for SyncAggregatorSelectionData {}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ and tree hash encoding/decoding tests for SyncAggregatorSelectionData.
    ssz_and_tree_hash_tests!(SyncAggregatorSelectionData);
}
