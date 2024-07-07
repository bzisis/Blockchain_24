use crate::*;
use serde::{Deserialize, Serialize};

/// Data structure representing proposer preparation information.
///
/// This structure is used to prepare the beacon node for potential proposers
/// by providing necessary information for proposing blocks on behalf of validators.
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct ProposerPreparationData {
    /// The index of the validator associated with this preparation data.
    #[serde(with = "serde_utils::quoted_u64")]
    pub validator_index: u64,

    /// The address of the fee recipient for block proposals.
    pub fee_recipient: Address,
}
