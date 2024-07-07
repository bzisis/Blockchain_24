use crate::*;

/// Represents a reference to a beacon committee.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct BeaconCommittee<'a> {
    /// Slot number associated with the committee.
    pub slot: Slot,
    /// Index of the committee.
    pub index: CommitteeIndex,
    /// Reference to the list of validator indices in the committee.
    pub committee: &'a [usize],
}

impl<'a> BeaconCommittee<'a> {
    /// Converts the `BeaconCommittee` into an `OwnedBeaconCommittee`.
    ///
    /// This function consumes the current `BeaconCommittee` and returns an owned version of it,
    /// where the committee field is converted into an owned `Vec<usize>`.
    ///
    /// # Returns
    ///
    /// An `OwnedBeaconCommittee` with all fields copied from the current `BeaconCommittee`.
    pub fn into_owned(self) -> OwnedBeaconCommittee {
        OwnedBeaconCommittee {
            slot: self.slot,
            index: self.index,
            committee: self.committee.to_vec(),
        }
    }
}

/// Represents an owned version of a beacon committee.
#[derive(arbitrary::Arbitrary, Default, Clone, Debug, PartialEq)]
pub struct OwnedBeaconCommittee {
    /// Slot number associated with the committee.
    pub slot: Slot,
    /// Index of the committee.
    pub index: CommitteeIndex,
    /// List of validator indices in the committee.
    pub committee: Vec<usize>,
}
