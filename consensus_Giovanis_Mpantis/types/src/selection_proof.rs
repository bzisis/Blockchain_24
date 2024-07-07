use crate::{
    ChainSpec, Domain, EthSpec, Fork, Hash256, PublicKey, SecretKey, Signature, SignedRoot, Slot,
};
use ethereum_hashing::hash;
use safe_arith::{ArithError, SafeArith};
use ssz::Encode;
use std::cmp;

/// A proof used to determine validator selection in the Ethereum 2.0 protocol.
#[derive(arbitrary::Arbitrary, PartialEq, Debug, Clone)]
pub struct SelectionProof(Signature);

impl SelectionProof {
    /// Creates a new `SelectionProof` for a given slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot for which the proof is generated.
    /// * `secret_key` - The secret key of the validator.
    /// * `fork` - The current fork parameters.
    /// * `genesis_validators_root` - The root of the genesis validators.
    /// * `spec` - The chain specification.
    ///
    /// # Returns
    ///
    /// A new `SelectionProof`.
    pub fn new<E: EthSpec>(
        slot: Slot,
        secret_key: &SecretKey,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> Self {
        let domain = spec.get_domain(
            slot.epoch(E::slots_per_epoch()),
            Domain::SelectionProof,
            fork,
            genesis_validators_root,
        );
        let message = slot.signing_root(domain);

        Self(secret_key.sign(message))
    }

    /// Returns the "modulo" used for determining if a `SelectionProof` elects an aggregator.
    ///
    /// # Arguments
    ///
    /// * `committee_len` - The length of the committee.
    /// * `spec` - The chain specification.
    ///
    /// # Returns
    ///
    /// The modulo value as a `Result<u64, ArithError>`.
    pub fn modulo(committee_len: usize, spec: &ChainSpec) -> Result<u64, ArithError> {
        Ok(cmp::max(
            1,
            (committee_len as u64).safe_div(spec.target_aggregators_per_committee)?,
        ))
    }

    /// Determines if the selection proof indicates that the validator is an aggregator.
    ///
    /// # Arguments
    ///
    /// * `committee_len` - The length of the committee.
    /// * `spec` - The chain specification.
    ///
    /// # Returns
    ///
    /// `true` if the validator is an aggregator, otherwise `false`.
    pub fn is_aggregator(
        &self,
        committee_len: usize,
        spec: &ChainSpec,
    ) -> Result<bool, ArithError> {
        self.is_aggregator_from_modulo(Self::modulo(committee_len, spec)?)
    }

    /// Determines if the selection proof indicates that the validator is an aggregator given a specific modulo.
    ///
    /// # Arguments
    ///
    /// * `modulo` - The modulo value.
    ///
    /// # Returns
    ///
    /// `true` if the validator is an aggregator, otherwise `false`.
    pub fn is_aggregator_from_modulo(&self, modulo: u64) -> Result<bool, ArithError> {
        let signature_hash = hash(&self.0.as_ssz_bytes());
        let signature_hash_int = u64::from_le_bytes(
            signature_hash
                .get(0..8)
                .expect("hash is 32 bytes")
                .try_into()
                .expect("first 8 bytes of signature should always convert to fixed array"),
        );

        signature_hash_int.safe_rem(modulo).map(|rem| rem == 0)
    }

    /// Verifies the selection proof.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot for which the proof is verified.
    /// * `pubkey` - The public key of the validator.
    /// * `fork` - The current fork parameters.
    /// * `genesis_validators_root` - The root of the genesis validators.
    /// * `spec` - The chain specification.
    ///
    /// # Returns
    ///
    /// `true` if the proof is valid, otherwise `false`.
    pub fn verify<E: EthSpec>(
        &self,
        slot: Slot,
        pubkey: &PublicKey,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> bool {
        let domain = spec.get_domain(
            slot.epoch(E::slots_per_epoch()),
            Domain::SelectionProof,
            fork,
            genesis_validators_root,
        );
        let message = slot.signing_root(domain);

        self.0.verify(pubkey, message)
    }
}

impl From<SelectionProof> for Signature {
    /// Converts a `SelectionProof` into a `Signature`.
    ///
    /// # Arguments
    ///
    /// * `from` - The `SelectionProof` to be converted.
    ///
    /// # Returns
    ///
    /// The corresponding `Signature`.
    fn from(from: SelectionProof) -> Signature {
        from.0
    }
}

impl From<Signature> for SelectionProof {
    /// Converts a `Signature` into a `SelectionProof`.
    ///
    /// # Arguments
    ///
    /// * `sig` - The `Signature` to be converted.
    ///
    /// # Returns
    ///
    /// The corresponding `SelectionProof`.
    fn from(sig: Signature) -> Self {
        Self(sig)
    }
}
