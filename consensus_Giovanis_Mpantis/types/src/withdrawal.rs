use crate::test_utils::TestRandom;
use crate::*;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a withdrawal object.
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
pub struct Withdrawal {
    /// The index of the withdrawal.
    #[serde(with = "serde_utils::quoted_u64")]
    pub index: u64,

    /// The index of the validator associated with the withdrawal.
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,

    /// The address where the withdrawal is made.
    pub address: Address,

    /// The amount of the withdrawal.
    #[serde(with = "serde_utils::quoted_u64")]
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // This macro generates SSZ and TreeHash tests for Withdrawal.
    ssz_and_tree_hash_tests!(Withdrawal);
}
