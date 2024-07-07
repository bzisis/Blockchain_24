use crate::{
    test_utils::TestRandom, ChainSpec, Domain, Epoch, ForkName, Hash256, SecretKey, SignedRoot,
    SignedVoluntaryExit,
};

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// An exit voluntarily submitted by a validator who wishes to withdraw.
///
/// This struct represents a voluntary exit as specified in Spec v0.12.1.
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
pub struct VoluntaryExit {
    /// Earliest epoch when the voluntary exit can be processed.
    pub epoch: Epoch,
    /// Index of the validator submitting the voluntary exit.
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,
}

impl SignedRoot for VoluntaryExit {}

impl VoluntaryExit {
    /// Signs the voluntary exit using a secret key, genesis validators root, and chain specification.
    ///
    /// Returns a `SignedVoluntaryExit` containing the signed message and signature.
    pub fn sign(
        self,
        secret_key: &SecretKey,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> SignedVoluntaryExit {
        // Determine the fork name and version based on the exit epoch.
        let fork_name = spec.fork_name_at_epoch(self.epoch);
        let fork_version = match fork_name {
            ForkName::Base | ForkName::Altair | ForkName::Bellatrix | ForkName::Capella => {
                spec.fork_version_for_name(fork_name)
            }
            // EIP-7044
            ForkName::Deneb | ForkName::Electra => spec.fork_version_for_name(ForkName::Capella),
        };
        // Compute the domain for the voluntary exit message.
        let domain =
            spec.compute_domain(Domain::VoluntaryExit, fork_version, genesis_validators_root);

        // Compute the signing root of the voluntary exit.
        let message = self.signing_root(domain);
        // Sign the message using the provided secret key.
        SignedVoluntaryExit {
            message: self,
            signature: secret_key.sign(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure the VoluntaryExit struct conforms to SSZ and tree hashing tests.
    ssz_and_tree_hash_tests!(VoluntaryExit);
}
