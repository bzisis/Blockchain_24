use crate::test_utils::TestRandom;
use crate::{ChainSpec, Domain, EthSpec, Fork, Hash256, SecretKey, Signature, SignedRoot, Slot};

use crate::slot_data::SlotData;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Represents a message from a sync committee, containing data related to a specific slot.
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct SyncCommitteeMessage {
    /// Slot number this message relates to.
    pub slot: Slot,
    /// Hash of the beacon block for which this message is created.
    pub beacon_block_root: Hash256,
    /// Index of the validator who created this message.
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,
    /// Signature by the validator over `beacon_block_root`.
    pub signature: Signature,
}

impl SyncCommitteeMessage {
    /// Constructs a new `SyncCommitteeMessage`.
    ///
    /// This method creates a new message for the given slot, beacon block root, validator index,
    /// using the provided secret key, fork information, genesis validators root, and chain specification.
    ///
    /// Equivalent to `get_sync_committee_message` from the specification.
    pub fn new<E: EthSpec>(
        slot: Slot,
        beacon_block_root: Hash256,
        validator_index: u64,
        secret_key: &SecretKey,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> Self {
        let epoch = slot.epoch(E::slots_per_epoch());
        let domain = spec.get_domain(epoch, Domain::SyncCommittee, fork, genesis_validators_root);
        let message = beacon_block_root.signing_root(domain);
        let signature = secret_key.sign(message);
        Self {
            slot,
            beacon_block_root,
            validator_index,
            signature,
        }
    }
}

impl SlotData for SyncCommitteeMessage {
    /// Returns the slot number associated with this sync committee message.
    fn get_slot(&self) -> Slot {
        self.slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(SyncCommitteeMessage);
}
