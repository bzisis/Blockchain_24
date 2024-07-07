use crate::*;
use serde::{Deserialize, Serialize};

/// Duty details for an attester.
///
/// This struct encapsulates the information required for an attester to perform their duty
/// regarding attestation at a specific slot and committee index.
#[derive(arbitrary::Arbitrary, Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AttestationDuty {
    /// The slot during which the attester must attest.
    pub slot: Slot,
    /// The index of this committee within the committees in `slot`.
    pub index: CommitteeIndex,
    /// The position of the attester within the committee.
    pub committee_position: usize,
    /// The total number of attesters in the committee.
    pub committee_len: usize,
    /// The committee count at `attestation_slot`.
    ///
    /// This field represents the total number of committees available at the `attestation_slot`.
    #[serde(with = "serde_utils::quoted_u64")]
    pub committees_at_slot: u64,
}
