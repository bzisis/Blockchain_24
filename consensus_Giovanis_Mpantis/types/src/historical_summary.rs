use crate::test_utils::TestRandom;
use crate::Unsigned;
use crate::{BeaconState, EthSpec, Hash256};
use cached_tree_hash::Error;
use cached_tree_hash::{int_log, CacheArena, CachedTreeHash, TreeHashCache};
use compare_fields_derive::CompareFields;
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::VariableList;
use test_random_derive::TestRandom;
use tree_hash::{mix_in_length, TreeHash, BYTES_PER_CHUNK};
use tree_hash_derive::TreeHash;

/// `HistoricalSummary` matches the components of the phase0 `HistoricalBatch`
/// making the two hash_tree_root-compatible. This struct is introduced into the beacon state
/// in the Capella hard fork.
///
/// [Capella Hard Fork Spec - HistoricalSummary](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/beacon-chain.md#historicalsummary)
#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
    CompareFields,
    Clone,
    Copy,
    Default,
    arbitrary::Arbitrary,
)]
pub struct HistoricalSummary {
    /// Root of the block summaries.
    pub block_summary_root: Hash256,

    /// Root of the state summaries.
    pub state_summary_root: Hash256,
}

impl HistoricalSummary {
    /// Constructs a new `HistoricalSummary` from a given `BeaconState`.
    pub fn new<E: EthSpec>(state: &BeaconState<E>) -> Self {
        Self {
            block_summary_root: state.block_roots().tree_hash_root(),
            state_summary_root: state.state_roots().tree_hash_root(),
        }
    }
}

/// Wrapper type allowing the implementation of `CachedTreeHash`.
#[derive(Debug)]
pub struct HistoricalSummaryCache<'a, N: Unsigned> {
    /// Inner variable list of `HistoricalSummary` instances.
    pub inner: &'a VariableList<HistoricalSummary, N>,
}

impl<'a, N: Unsigned> HistoricalSummaryCache<'a, N> {
    /// Creates a new `HistoricalSummaryCache` from the inner variable list.
    pub fn new(inner: &'a VariableList<HistoricalSummary, N>) -> Self {
        Self { inner }
    }

    /// Returns the length of the inner variable list.
    ///
    /// This method does not check if the list is empty.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, N: Unsigned> CachedTreeHash<TreeHashCache> for HistoricalSummaryCache<'a, N> {
    /// Creates a new tree hash cache for the `HistoricalSummaryCache`.
    ///
    /// Uses the provided `arena` to allocate cache nodes and calculates the logarithm of `N`
    /// as the depth of the tree.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> TreeHashCache {
        TreeHashCache::new(arena, int_log(N::to_usize()), self.len())
    }

    /// Recalculates the tree hash root for the `HistoricalSummaryCache`.
    ///
    /// Uses the provided `arena` and `cache` to compute the Merkle root of the cached tree
    /// nodes. It mixes in the length of the inner list for finalization.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut TreeHashCache,
    ) -> Result<Hash256, Error> {
        Ok(mix_in_length(
            &cache.recalculate_merkle_root(arena, leaf_iter(self.inner))?,
            self.len(),
        ))
    }
}

/// Iterator over `HistoricalSummary` instances, producing fixed-size byte arrays.
pub fn leaf_iter(
    values: &[HistoricalSummary],
) -> impl ExactSizeIterator<Item = [u8; BYTES_PER_CHUNK]> + '_ {
    values
        .iter()
        .map(|value| value.tree_hash_root())
        .map(Hash256::to_fixed_bytes)
}
