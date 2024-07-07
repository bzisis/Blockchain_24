//! Identifies each sync committee subnet by an integer identifier.
use crate::consts::altair::SYNC_COMMITTEE_SUBNET_COUNT;
use crate::EthSpec;
use lazy_static::lazy_static;
use safe_arith::{ArithError, SafeArith};
use serde::{Deserialize, Serialize};
use ssz_types::typenum::Unsigned;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::ops::{Deref, DerefMut};

lazy_static! {
    /// Precomputed strings representing Sync Subnet IDs from 0 to SYNC_COMMITTEE_SUBNET_COUNT - 1.
    static ref SYNC_SUBNET_ID_TO_STRING: Vec<String> = {
        let mut v = Vec::with_capacity(SYNC_COMMITTEE_SUBNET_COUNT as usize);

        for i in 0..SYNC_COMMITTEE_SUBNET_COUNT {
            v.push(i.to_string());
        }
        v
    };
}

/// Wrapper struct around `u64` representing a Sync Subnet ID.
#[derive(
    arbitrary::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
)]
#[serde(transparent)]
pub struct SyncSubnetId(#[serde(with = "serde_utils::quoted_u64")] u64);

/// Converts a `u64` Sync Subnet ID to its corresponding string representation.
pub fn sync_subnet_id_to_string(i: u64) -> &'static str {
    if i < SYNC_COMMITTEE_SUBNET_COUNT {
        SYNC_SUBNET_ID_TO_STRING
            .get(i as usize)
            .expect("index below SYNC_COMMITTEE_SUBNET_COUNT")
    } else {
        "sync subnet id out of range"
    }
}

impl SyncSubnetId {
    /// Creates a new `SyncSubnetId` from a `u64`.
    pub fn new(id: u64) -> Self {
        id.into()
    }

    /// Computes the required subnets to subscribe to given the sync committee indices.
    ///
    /// Returns a `HashSet` of `SyncSubnetId`.
    ///
    /// # Errors
    ///
    /// Returns an `ArithError` if the division by `E::SyncSubcommitteeSize` fails.
    pub fn compute_subnets_for_sync_committee<E: EthSpec>(
        sync_committee_indices: &[u64],
    ) -> Result<HashSet<Self>, ArithError> {
        let subcommittee_size = E::SyncSubcommitteeSize::to_u64();

        sync_committee_indices
            .iter()
            .map(|index| index.safe_div(subcommittee_size).map(Self::new))
            .collect()
    }
}

impl Display for SyncSubnetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl Deref for SyncSubnetId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SyncSubnetId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u64> for SyncSubnetId {
    fn from(x: u64) -> Self {
        Self(x)
    }
}

impl From<SyncSubnetId> for u64 {
    fn from(from: SyncSubnetId) -> u64 {
        from.0
    }
}

impl From<&SyncSubnetId> for u64 {
    fn from(from: &SyncSubnetId) -> u64 {
        from.0
    }
}

impl AsRef<str> for SyncSubnetId {
    fn as_ref(&self) -> &str {
        sync_subnet_id_to_string(self.0)
    }
}