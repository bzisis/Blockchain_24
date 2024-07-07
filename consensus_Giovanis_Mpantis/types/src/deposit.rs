use crate::test_utils::TestRandom;
use crate::*;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::typenum::U33;
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;
use ssz_types::FixedVector;
use types::DepositData; // Assuming DepositData is defined in the `types` module
use types::Hash256; // Assuming Hash256 is defined in the `types` module

pub const DEPOSIT_TREE_DEPTH: usize = 32;

/// A deposit to potentially become a beacon chain validator.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    PartialEq,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct Deposit {
    /// Merkle proof of inclusion of the deposit data in the deposit tree.
    pub proof: FixedVector<Hash256, U33>,
    /// The actual deposit data.
    pub data: DepositData,
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(Deposit);
}
