use crate::test_utils::TestRandom;
use crate::{Consolidation, Signature};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// A signed message containing a consolidation and its associated signature.
///
/// This struct represents a consolidation message along with a signature 
/// verifying its authenticity.
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
pub struct SignedConsolidation {
    /// The consolidation message.
    pub message: Consolidation,
    /// The signature of the consolidation message.
    pub signature: Signature,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the SSZ (Simple Serialize) and Tree Hash tests for `SignedConsolidation`.
    ssz_and_tree_hash_tests!(SignedConsolidation);
}
