use crate::test_utils::TestRandom;
use crate::*;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a change from BLS to execution for a validator.
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
pub struct BlsToExecutionChange {
    /// Validator index affected by the change.
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,
    /// Public key used for BLS operations.
    pub from_bls_pubkey: PublicKeyBytes,
    /// Execution address after the change.
    pub to_execution_address: Address,
}

impl SignedRoot for BlsToExecutionChange {}

impl BlsToExecutionChange {
    /// Signs the `BlsToExecutionChange` with the provided `secret_key`.
    ///
    /// Returns a `SignedBlsToExecutionChange` containing the original message and its signature.
    pub fn sign(
        self,
        secret_key: &SecretKey,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> SignedBlsToExecutionChange {
        let domain = spec.compute_domain(
            Domain::BlsToExecutionChange,
            spec.genesis_fork_version,
            genesis_validators_root,
        );
        let message = self.signing_root(domain);
        SignedBlsToExecutionChange {
            message: self,
            signature: secret_key.sign(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Automatically generates serialization, deserialization, and tree hashing tests.
    ssz_and_tree_hash_tests!(BlsToExecutionChange);
}
