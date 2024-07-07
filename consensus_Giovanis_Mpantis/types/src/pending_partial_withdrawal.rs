use crate::{test_utils::TestRandom, Epoch};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a pending partial withdrawal with an index, amount, and withdrawable epoch.
///
/// This struct is used to represent pending partial withdrawals of funds or other resources,
/// specifying the index, amount, and epoch when the withdrawal becomes withdrawable.
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
pub struct PendingPartialWithdrawal {
    /// Index associated with the pending partial withdrawal.
    #[serde(with = "serde_utils::quoted_u64")]
    pub index: u64,
    /// Amount associated with the pending partial withdrawal.
    #[serde(with = "serde_utils::quoted_u64")]
    pub amount: u64,
    /// Epoch when the partial withdrawal becomes withdrawable.
    pub withdrawable_epoch: Epoch,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ and TreeHash implementations are correct for PendingPartialWithdrawal.
    ssz_and_tree_hash_tests!(PendingPartialWithdrawal);
}
