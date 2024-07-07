use crate::{Hash256, PublicKeyBytes, Signature};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;
use types::SignedRoot; // Assuming types are defined in the `types` module

/// Represents a receipt of a deposit made to the beacon chain.
///
/// This struct contains information about the deposit, including the public key, withdrawal
/// credentials, amount deposited, signature, and index.
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
pub struct DepositReceipt {
    /// Public key of the validator.
    pub pubkey: PublicKeyBytes,
    /// Hash of the validator's withdrawal credentials.
    pub withdrawal_credentials: Hash256,
    /// The amount of ETH deposited.
    #[serde(with = "serde_utils::quoted_u64")]
    pub amount: u64,
    /// Signature of the deposit data.
    pub signature: Signature,
    /// Index of the deposit in the Merkle tree or another sequence identifier.
    #[serde(with = "serde_utils::quoted_u64")]
    pub index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(DepositReceipt);
}
