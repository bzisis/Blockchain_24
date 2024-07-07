use crate::test_utils::TestRandom;
use crate::{Address, PublicKeyBytes};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a withdrawal request from the execution layer.
///
/// This struct encapsulates data related to a withdrawal request, including source address,
/// validator public key, and withdrawal amount.
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
pub struct ExecutionLayerWithdrawalRequest {
    /// Source address from which the withdrawal is made.
    pub source_address: Address,

    /// Public key of the validator associated with the withdrawal.
    pub validator_pubkey: PublicKeyBytes,

    /// Amount to be withdrawn (serialized as a quoted u64).
    #[serde(with = "serde_utils::quoted_u64")]
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Automatically generates SSZ and Merkle tree hash tests for ExecutionLayerWithdrawalRequest.
    ssz_and_tree_hash_tests!(ExecutionLayerWithdrawalRequest);
}
