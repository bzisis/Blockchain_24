use crate::test_utils::TestRandom;
use crate::*;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Historical block and state roots.
///
/// This struct represents the historical block and state roots according to the Ethereum
/// 2.0 specification version 0.12.1.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    arbitrary::Arbitrary,
)]
#[serde(bound = "")]
#[arbitrary(bound = "E: EthSpec")]
pub struct HistoricalBatch<E: EthSpec> {
    /// Block roots stored in a vector.
    #[test_random(default)]
    pub block_roots: Vector<Hash256, E::SlotsPerHistoricalRoot>,

    /// State roots stored in a vector.
    #[test_random(default)]
    pub state_roots: Vector<Hash256, E::SlotsPerHistoricalRoot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Alias for HistoricalBatch using the MainnetEthSpec.
    pub type FoundationHistoricalBatch = HistoricalBatch<MainnetEthSpec>;

    // Test suite for ssz and tree hash implementations.
    ssz_and_tree_hash_tests!(FoundationHistoricalBatch);
}
