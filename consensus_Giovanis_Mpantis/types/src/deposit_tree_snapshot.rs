use crate::*;
use ethereum_hashing::{hash32_concat, ZERO_HASHES};
use int_to_bytes::int_to_bytes32;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use test_utils::TestRandom;

/// Represents a finalized execution block in the beacon chain.
#[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TestRandom)]
pub struct FinalizedExecutionBlock {
    /// Root hash of the deposits included in the block.
    pub deposit_root: Hash256,
    /// Number of deposits included in the block.
    pub deposit_count: u64,
    /// Hash of the block.
    pub block_hash: Hash256,
    /// Height of the block.
    pub block_height: u64,
}

impl From<&DepositTreeSnapshot> for FinalizedExecutionBlock {
    /// Converts a `DepositTreeSnapshot` into a `FinalizedExecutionBlock`.
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The `DepositTreeSnapshot` to convert.
    ///
    /// # Returns
    ///
    /// A `FinalizedExecutionBlock` corresponding to the given `snapshot`.
    fn from(snapshot: &DepositTreeSnapshot) -> Self {
        Self {
            deposit_root: snapshot.deposit_root,
            deposit_count: snapshot.deposit_count,
            block_hash: snapshot.execution_block_hash,
            block_height: snapshot.execution_block_height,
        }
    }
}

/// Represents a snapshot of the deposit tree in the beacon chain.
#[derive(Encode, Decode, Deserialize, Serialize, Clone, Debug, PartialEq, TestRandom)]
pub struct DepositTreeSnapshot {
    /// Hashes of finalized blocks.
    pub finalized: Vec<Hash256>,
    /// Root hash of the deposits included in the snapshot.
    pub deposit_root: Hash256,
    /// Number of deposits included in the snapshot.
    #[serde(with = "serde_utils::quoted_u64")]
    pub deposit_count: u64,
    /// Hash of the execution block.
    pub execution_block_hash: Hash256,
    /// Height of the execution block.
    #[serde(with = "serde_utils::quoted_u64")]
    pub execution_block_height: u64,
}

impl Default for DepositTreeSnapshot {
    /// Creates a default `DepositTreeSnapshot`.
    ///
    /// The default snapshot has an empty `finalized` vector, zeroed `deposit_root` and `execution_block_hash`,
    /// and deposit and execution block heights set to zero.
    fn default() -> Self {
        let mut result = Self {
            finalized: vec![],
            deposit_root: Hash256::default(),
            deposit_count: 0,
            execution_block_hash: Hash256::zero(),
            execution_block_height: 0,
        };
        // properly set the empty deposit root
        result.deposit_root = result.calculate_root().unwrap();
        result
    }
}

impl DepositTreeSnapshot {
    /// Calculates the deposit tree root hash from the hashes in the snapshot.
    ///
    /// Returns `Some(Hash256)` if calculation succeeds, `None` otherwise.
    pub fn calculate_root(&self) -> Option<Hash256> {
        let mut size = self.deposit_count;
        let mut index = self.finalized.len();
        let mut deposit_root = [0; 32];
        for height in 0..DEPOSIT_TREE_DEPTH {
            deposit_root = if (size & 1) == 1 {
                index = index.checked_sub(1)?;
                hash32_concat(self.finalized.get(index)?.as_bytes(), &deposit_root)
            } else {
                hash32_concat(&deposit_root, ZERO_HASHES.get(height)?)
            };
            size /= 2;
        }
        // add mix-in-length
        deposit_root = hash32_concat(&deposit_root, &int_to_bytes32(self.deposit_count));

        Some(Hash256::from_slice(&deposit_root))
    }

    /// Checks if the `DepositTreeSnapshot` is valid.
    ///
    /// Returns `true` if the calculated deposit root matches the stored deposit root, `false` otherwise.
    pub fn is_valid(&self) -> bool {
        self.calculate_root()
            .map_or(false, |calculated| self.deposit_root == calculated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    ssz_tests!(DepositTreeSnapshot);
}
