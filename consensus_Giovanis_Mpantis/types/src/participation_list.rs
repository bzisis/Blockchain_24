#![allow(clippy::arithmetic_side_effects)]

use crate::{Hash256, ParticipationFlags, Unsigned, VariableList};
use cached_tree_hash::{int_log, CacheArena, CachedTreeHash, Error, TreeHashCache};
use tree_hash::{mix_in_length, BYTES_PER_CHUNK};

/// Wrapper type allowing the implementation of `CachedTreeHash`.
#[derive(Debug)]
pub struct ParticipationList<'a, N: Unsigned> {
    pub inner: &'a VariableList<ParticipationFlags, N>,
}

impl<'a, N: Unsigned> ParticipationList<'a, N> {
    /// Creates a new `ParticipationList` wrapper around a `VariableList` of `ParticipationFlags`.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate_name::{ParticipationList, VariableList};
    /// use typenum::U10; // Example Unsigned type, could be any Unsigned type.
    ///
    /// let flags_list: VariableList<ParticipationFlags, U10> = VariableList::new();
    /// let participation_list = ParticipationList::new(&flags_list);
    /// ```
    pub fn new(inner: &'a VariableList<ParticipationFlags, N>) -> Self {
        Self { inner }
    }
}

impl<'a, N: Unsigned> CachedTreeHash<TreeHashCache> for ParticipationList<'a, N> {
    /// Creates a new `TreeHashCache` for the `ParticipationList`.
    ///
    /// # Arguments
    ///
    /// * `arena` - A mutable reference to a `CacheArena`, which is used for caching tree hash computations.
    ///
    /// # Returns
    ///
    /// A new `TreeHashCache` instance configured for `ParticipationList`.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> TreeHashCache {
        TreeHashCache::new(
            arena,
            int_log(N::to_usize() / BYTES_PER_CHUNK),
            leaf_count(self.inner.len()),
        )
    }

    /// Recalculates the Merkle root hash for the `ParticipationList`.
    ///
    /// # Arguments
    ///
    /// * `arena` - A mutable reference to a `CacheArena`, which is used for caching tree hash computations.
    /// * `cache` - A mutable reference to a `TreeHashCache`, which stores cached hashes and tree structure.
    ///
    /// # Returns
    ///
    /// The recalculated Merkle root hash of the `ParticipationList`.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there is an issue during the Merkle root calculation.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut TreeHashCache,
    ) -> Result<Hash256, Error> {
        Ok(mix_in_length(
            &cache.recalculate_merkle_root(arena, leaf_iter(self.inner))?,
            self.inner.len(),
        ))
    }
}

/// Calculates the number of leaf nodes required for hashing `len` items.
///
/// # Arguments
///
/// * `len` - The number of items to be hashed.
///
/// # Returns
///
/// The number of leaf nodes required for hashing `len` items.
pub fn leaf_count(len: usize) -> usize {
    (len + BYTES_PER_CHUNK - 1) / BYTES_PER_CHUNK
}

/// Converts an array of `ParticipationFlags` into an iterator of fixed-size byte chunks.
///
/// # Arguments
///
/// * `values` - Slice of `ParticipationFlags` to be converted into byte chunks.
///
/// # Returns
///
/// An iterator over fixed-size byte chunks, each representing a subset of `ParticipationFlags`.
pub fn leaf_iter(
    values: &[ParticipationFlags],
) -> impl ExactSizeIterator<Item = [u8; BYTES_PER_CHUNK]> + '_ {
    values.chunks(BYTES_PER_CHUNK).map(|xs| {
        // Zero-pad chunks on the right.
        let mut chunk = [0u8; BYTES_PER_CHUNK];
        for (byte, x) in chunk.iter_mut().zip(xs) {
            *byte = x.into_u8();
        }
        chunk
    })
}
