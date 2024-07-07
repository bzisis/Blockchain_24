use crate::{test_utils::TestRandom, EthSpec, IndexedAttestation};

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents two conflicting attestations.
///
/// This struct encapsulates two conflicting attestations, which are used to demonstrate
/// slashing conditions in the Ethereum 2.0 Beacon Chain.
///
/// Spec v0.12.1
#[derive(
    Derivative,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    arbitrary::Arbitrary,
)]
#[derivative(PartialEq, Eq, Hash(bound = "E: EthSpec"))]
#[serde(bound = "E: EthSpec")]
#[arbitrary(bound = "E: EthSpec")]
pub struct AttesterSlashing<E: EthSpec> {
    /// The first attestation involved in the slashing condition.
    pub attestation_1: IndexedAttestation<E>,
    /// The second attestation involved in the slashing condition.
    pub attestation_2: IndexedAttestation<E>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    // Test for SSZ and tree hashing traits.
    ssz_and_tree_hash_tests!(AttesterSlashing<MainnetEthSpec>);
}
