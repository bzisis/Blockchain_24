use crate::test_utils::TestRandom;
use crate::{AttestationData, BitList, EthSpec};

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// An attestation that has been included in the state but not yet fully processed.
///
/// This struct represents an attestation in the Ethereum 2.0 specification, version 0.12.1.
///
/// # Type Parameters
///
/// * `E`: Type parameter representing the Ethereum specification (`EthSpec`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    arbitrary::Arbitrary,
)]
#[serde(bound = "")]
#[arbitrary(bound = "E: EthSpec")]
pub struct PendingAttestation<E: EthSpec> {
    /// Bitlist representing the aggregation bits of the validators.
    pub aggregation_bits: BitList<E::MaxValidatorsPerCommittee>,
    /// Attestation data associated with this pending attestation.
    pub data: AttestationData,
    /// Delay before inclusion of this attestation in a block.
    #[serde(with = "serde_utils::quoted_u64")]
    pub inclusion_delay: u64,
    /// Index of the proposer who included this attestation.
    #[serde(with = "serde_utils::quoted_u64")]
    pub proposer_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    // Ensure SSZ and TreeHash implementations are correct for PendingAttestation.
    ssz_and_tree_hash_tests!(PendingAttestation<MainnetEthSpec>);
}
