use crate::test_utils::TestRandom;
use crate::Epoch;

use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;

/// Specifies a fork of the `BeaconChain`, to prevent replay attacks.
///
/// This struct represents a fork of the blockchain, defining previous and current versions
/// alongside an epoch marking when the fork occurred.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct Fork {
    /// Previous version identifier, serialized as hex.
    #[serde(with = "serde_utils::bytes_4_hex")]
    pub previous_version: [u8; 4],
    /// Current version identifier, serialized as hex.
    #[serde(with = "serde_utils::bytes_4_hex")]
    pub current_version: [u8; 4],
    /// Epoch when this fork took place.
    pub epoch: Epoch,
}

impl Fork {
    /// Returns the fork version for a given epoch.
    ///
    /// Returns the previous version if the provided epoch is less than the fork's epoch.
    /// Otherwise, returns the current version.
    ///
    /// Spec v0.12.1
    ///
    /// # Arguments
    ///
    /// * `epoch` - The epoch for which to retrieve the fork version.
    ///
    /// # Returns
    ///
    /// An array of 4 bytes representing the fork version.
    pub fn get_fork_version(&self, epoch: Epoch) -> [u8; 4] {
        if epoch < self.epoch {
            return self.previous_version;
        }
        self.current_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure SSZ serialization and tree hashing tests are generated for Fork.
    ssz_and_tree_hash_tests!(Fork);

    #[test]
    fn get_fork_version() {
        let previous_version = [1; 4];
        let current_version = [2; 4];
        let epoch = Epoch::new(10);

        let fork = Fork {
            previous_version,
            current_version,
            epoch,
        };

        assert_eq!(fork.get_fork_version(epoch - 1), previous_version);
        assert_eq!(fork.get_fork_version(epoch), current_version);
        assert_eq!(fork.get_fork_version(epoch + 1), current_version);
    }
}
