use crate::{
    test_utils::TestRandom, BeaconBlockHeader, ChainSpec, Domain, EthSpec, Fork, Hash256,
    PublicKey, Signature, SignedRoot,
};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// A signed header of a `BeaconBlock`.
///
/// This struct represents a signed header of a BeaconBlock in the Ethereum 2.0 specification.
/// It contains the header message and the signature. This struct follows the specification version v0.12.1.
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct SignedBeaconBlockHeader {
    /// The header message of the BeaconBlock.
    pub message: BeaconBlockHeader,
    
    /// The signature of the header.
    pub signature: Signature,
}

impl SignedBeaconBlockHeader {
    /// Verifies that the block header was signed by the given public key.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - A reference to the public key that is supposed to have signed the header.
    /// * `fork` - The current fork of the Ethereum 2.0 chain.
    /// * `genesis_validators_root` - The root hash of the genesis validators.
    /// * `spec` - The chain specification parameters.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the signature is valid, otherwise returns `false`.
    ///
    /// # Type Parameters
    ///
    /// * `E` - A type that implements the `EthSpec` trait, representing the Ethereum 2.0 specification.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use your_crate::{SignedBeaconBlockHeader, PublicKey, Fork, Hash256, ChainSpec};
    /// use your_crate::types::EthSpec;
    ///
    /// let header = SignedBeaconBlockHeader { ... };
    /// let pubkey = PublicKey::from_bytes(&[...]).unwrap();
    /// let fork = Fork { ... };
    /// let genesis_validators_root = Hash256::from_slice(&[...]);
    /// let spec = ChainSpec::mainnet();
    ///
    /// let is_valid = header.verify_signature::<EthSpec>(&pubkey, &fork, genesis_validators_root, &spec);
    /// assert!(is_valid);
    /// ```
    pub fn verify_signature<E: EthSpec>(
        &self,
        pubkey: &PublicKey,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> bool {
        let domain = spec.get_domain(
            self.message.slot.epoch(E::slots_per_epoch()),
            Domain::BeaconProposer,
            fork,
            genesis_validators_root,
        );

        let message = self.message.signing_root(domain);

        self.signature.verify(pubkey, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests for the `SignedBeaconBlockHeader` struct.
    ///
    /// This macro generates SSZ and tree hash tests for the `SignedBeaconBlockHeader` struct.
    ssz_and_tree_hash_tests!(SignedBeaconBlockHeader);
}
