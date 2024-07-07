use crate::test_utils::TestRandom;
use crate::*;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;
use types::{DepositMessage, PublicKeyBytes, SignatureBytes}; // Assuming types are defined in the `types` module
use types::Hash256; // Assuming Hash256 is defined in the `types` module
use types::SecretKey; // Assuming SecretKey is defined in the `types` module
use types::ChainSpec; // Assuming ChainSpec is defined in the `types` module

/// The data supplied by the user to the deposit contract.
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
pub struct DepositData {
    /// Public key of the validator.
    pub pubkey: PublicKeyBytes,
    /// Hash of the validator's withdrawal credentials.
    pub withdrawal_credentials: Hash256,
    /// The amount of ETH deposited.
    #[serde(with = "serde_utils::quoted_u64")]
    pub amount: u64,
    /// Signature of the deposit data.
    pub signature: SignatureBytes,
}

impl DepositData {
    /// Create a `DepositMessage` corresponding to this `DepositData`, for signature verification.
    ///
    /// Spec v0.12.1
    pub fn as_deposit_message(&self) -> DepositMessage {
        DepositMessage {
            pubkey: self.pubkey,
            withdrawal_credentials: self.withdrawal_credentials,
            amount: self.amount,
        }
    }

    /// Generate the signature for a given `DepositData` using the provided `secret_key` and `spec`.
    ///
    /// Spec v0.12.1
    ///
    /// # Arguments
    ///
    /// * `secret_key` - The secret key used for signing.
    /// * `spec` - The chain specification defining the domain for the signature.
    ///
    /// # Returns
    ///
    /// The signature bytes.
    pub fn create_signature(&self, secret_key: &SecretKey, spec: &ChainSpec) -> SignatureBytes {
        let domain = spec.get_deposit_domain();
        let msg = self.as_deposit_message().signing_root(domain);

        SignatureBytes::from(secret_key.sign(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(DepositData);
}
