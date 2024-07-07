use crate::Epoch;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};

/// A sync committee subscription created when a validator subscribes to sync committee subnets to perform
/// sync committee duties.
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Encode, Decode)]
pub struct SyncCommitteeSubscription {
    /// The validator's index.
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,
    
    /// The indices of sync committees the validator is subscribed to.
    #[serde(with = "serde_utils::quoted_u64_vec")]
    pub sync_committee_indices: Vec<u64>,
    
    /// The epoch until which this subscription is valid.
    pub until_epoch: Epoch,
}
