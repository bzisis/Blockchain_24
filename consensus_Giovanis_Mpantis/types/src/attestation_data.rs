use crate::test_utils::TestRandom;
use crate::{Checkpoint, Hash256, SignedRoot, Slot};

use crate::slot_data::SlotData;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// The data upon which an attestation is based.
///
/// This struct represents the core data that forms an attestation in the Ethereum 2.0 Beacon Chain.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    Default,
)]
pub struct AttestationData {
    /// The slot in which the attestation was created.
    pub slot: Slot,
    /// The index of the attestation.
    #[serde(with = "serde_utils::quoted_u64")]
    pub index: u64,

    /// Root of the beacon block that the attester is voting for (LMD GHOST vote).
    pub beacon_block_root: Hash256,

    /// Source checkpoint (FFG Vote).
    pub source: Checkpoint,
    /// Target checkpoint (FFG Vote).
    pub target: Checkpoint,
}

impl SignedRoot for AttestationData {}

impl SlotData for AttestationData {
    /// Returns the slot associated with this attestation data.
    fn get_slot(&self) -> Slot {
        self.slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Implementing tests for SSZ and tree hashing traits.
    ssz_and_tree_hash_tests!(AttestationData);
}
