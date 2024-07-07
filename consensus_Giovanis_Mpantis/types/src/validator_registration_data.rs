use crate::*;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;

/// Validator registration data, containing information required for registering a validator.
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct SignedValidatorRegistrationData {
    /// The main data structure of the registration, including details about the validator.
    pub message: ValidatorRegistrationData,
    /// Signature of the registration data.
    pub signature: Signature,
}

/// Data structure representing validator registration details.
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Encode, Decode, TreeHash)]
pub struct ValidatorRegistrationData {
    /// The address to which fees are directed.
    pub fee_recipient: Address,
    /// Gas limit for the registration transaction.
    #[serde(with = "serde_utils::quoted_u64")]
    pub gas_limit: u64,
    /// Timestamp indicating when the registration was initiated.
    #[serde(with = "serde_utils::quoted_u64")]
    pub timestamp: u64,
    /// Public key of the validator.
    pub pubkey: PublicKeyBytes,
}

/// Implementing the `SignedRoot` trait for `ValidatorRegistrationData`.
impl SignedRoot for ValidatorRegistrationData {}

impl SignedValidatorRegistrationData {
    /// Verifies the signature of the `SignedValidatorRegistrationData` using the given `ChainSpec`.
    ///
    /// Returns `true` if the signature is valid for the given `ChainSpec`, otherwise `false`.
    pub fn verify_signature(&self, spec: &ChainSpec) -> bool {
        self.message
            .pubkey
            .decompress()
            .map(|pubkey| {
                let domain = spec.get_builder_domain();
                let message = self.message.signing_root(domain);
                self.signature.verify(&pubkey, message)
            })
            .unwrap_or(false)
    }
}
