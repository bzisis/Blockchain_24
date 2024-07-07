use crate::test_utils::TestRandom;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a pending balance deposit with an index and an amount.
///
/// This struct is used to store information about pending balance deposits.
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
pub struct PendingBalanceDeposit {
    /// Index associated with the pending balance deposit.
    #[serde(with = "serde_utils::quoted_u64")]
    pub index: u64,
    /// Amount associated with the pending balance deposit.
    #[serde(with = "serde_utils::quoted_u64")]
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ and TreeHash implementations are correct for PendingBalanceDeposit.
    ssz_and_tree_hash_tests!(PendingBalanceDeposit);
}
