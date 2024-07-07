use crate::{CacheArena, CachedTreeHash, Error, Hash256, TreeHashCache};
use ssz_types::{typenum::Unsigned, FixedVector, VariableList};
use std::mem::size_of;
use tree_hash::{mix_in_length, BYTES_PER_CHUNK};

/// Compute ceil(log(n))
///
/// This function computes the smallest number of bits `d` such that `n <= 2^d`.
///
/// # Arguments
///
/// * `n` - The input number.
///
/// # Returns
///
/// * The ceiling of the logarithm base 2 of the input number.
pub fn int_log(n: usize) -> usize {
    match n.checked_next_power_of_two() {
        Some(x) => x.trailing_zeros() as usize,
        None => 8 * std::mem::size_of::<usize>(),
    }
}

/// Compute the number of leaves needed for a given length of `Hash256` values.
///
/// # Arguments
///
/// * `len` - The number of `Hash256` values.
///
/// # Returns
///
/// * The number of leaves.
pub fn hash256_leaf_count(len: usize) -> usize {
    len
}

/// Compute the number of leaves needed for a given length of `u64` values.
///
/// # Arguments
///
/// * `len` - The number of `u64` values.
///
/// # Returns
///
/// * The number of leaves.
pub fn u64_leaf_count(len: usize) -> usize {
    let type_size = size_of::<u64>();
    let vals_per_chunk = BYTES_PER_CHUNK / type_size;

    (len + vals_per_chunk - 1) / vals_per_chunk
}

/// Create an iterator over `Hash256` values, yielding chunks of `BYTES_PER_CHUNK` bytes.
///
/// # Arguments
///
/// * `values` - A slice of `Hash256` values.
///
/// # Returns
///
/// * An iterator over `[u8; BYTES_PER_CHUNK]` chunks.
pub fn hash256_iter(
    values: &[Hash256],
) -> impl ExactSizeIterator<Item = [u8; BYTES_PER_CHUNK]> + '_ {
    values.iter().copied().map(Hash256::to_fixed_bytes)
}

/// Create an iterator over `u64` values, yielding chunks of `BYTES_PER_CHUNK` bytes.
///
/// # Arguments
///
/// * `values` - A slice of `u64` values.
///
/// # Returns
///
/// * An iterator over `[u8; BYTES_PER_CHUNK]` chunks.
pub fn u64_iter(values: &[u64]) -> impl ExactSizeIterator<Item = [u8; BYTES_PER_CHUNK]> + '_ {
    let type_size = size_of::<u64>();
    let vals_per_chunk = BYTES_PER_CHUNK / type_size;
    values.chunks(vals_per_chunk).map(move |xs| {
        xs.iter().map(|x| x.to_le_bytes()).enumerate().fold(
            [0; BYTES_PER_CHUNK],
            |mut chunk, (i, x_bytes)| {
                chunk[i * type_size..(i + 1) * type_size].copy_from_slice(&x_bytes);
                chunk
            },
        )
    })
}

impl<N: Unsigned> CachedTreeHash<TreeHashCache> for FixedVector<Hash256, N> {
    /// Create a new `TreeHashCache` for the `FixedVector`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    ///
    /// # Returns
    ///
    /// * A new `TreeHashCache`.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> TreeHashCache {
        TreeHashCache::new(
            arena,
            int_log(N::to_usize()),
            hash256_leaf_count(self.len()),
        )
    }

    /// Recalculate the tree hash root for the `FixedVector`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    /// * `cache` - The tree hash cache.
    ///
    /// # Returns
    ///
    /// * The recalculated `Hash256` root or an `Error`.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut TreeHashCache,
    ) -> Result<Hash256, Error> {
        cache.recalculate_merkle_root(arena, hash256_iter(self))
    }
}

impl<N: Unsigned> CachedTreeHash<TreeHashCache> for FixedVector<u64, N> {
    /// Create a new `TreeHashCache` for the `FixedVector`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    ///
    /// # Returns
    ///
    /// * A new `TreeHashCache`.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> TreeHashCache {
        let vals_per_chunk = BYTES_PER_CHUNK / size_of::<u64>();
        TreeHashCache::new(
            arena,
            int_log(N::to_usize() / vals_per_chunk),
            u64_leaf_count(self.len()),
        )
    }

    /// Recalculate the tree hash root for the `FixedVector`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    /// * `cache` - The tree hash cache.
    ///
    /// # Returns
    ///
    /// * The recalculated `Hash256` root or an `Error`.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut TreeHashCache,
    ) -> Result<Hash256, Error> {
        cache.recalculate_merkle_root(arena, u64_iter(self))
    }
}

impl<N: Unsigned> CachedTreeHash<TreeHashCache> for VariableList<Hash256, N> {
    /// Create a new `TreeHashCache` for the `VariableList`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    ///
    /// # Returns
    ///
    /// * A new `TreeHashCache`.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> TreeHashCache {
        TreeHashCache::new(
            arena,
            int_log(N::to_usize()),
            hash256_leaf_count(self.len()),
        )
    }

    /// Recalculate the tree hash root for the `VariableList`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    /// * `cache` - The tree hash cache.
    ///
    /// # Returns
    ///
    /// * The recalculated `Hash256` root or an `Error`.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut TreeHashCache,
    ) -> Result<Hash256, Error> {
        Ok(mix_in_length(
            &cache.recalculate_merkle_root(arena, hash256_iter(self))?,
            self.len(),
        ))
    }
}

impl<N: Unsigned> CachedTreeHash<TreeHashCache> for VariableList<u64, N> {
    /// Create a new `TreeHashCache` for the `VariableList`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    ///
    /// # Returns
    ///
    /// * A new `TreeHashCache`.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> TreeHashCache {
        let vals_per_chunk = BYTES_PER_CHUNK / size_of::<u64>();
        TreeHashCache::new(
            arena,
            int_log(N::to_usize() / vals_per_chunk),
            u64_leaf_count(self.len()),
        )
    }

    /// Recalculate the tree hash root for the `VariableList`.
    ///
    /// # Arguments
    ///
    /// * `arena` - The cache arena.
    /// * `cache` - The tree hash cache.
    ///
    /// # Returns
    ///
    /// * The recalculated `Hash256` root or an `Error`.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut TreeHashCache,
    ) -> Result<Hash256, Error> {
        Ok(mix_in_length(
            &cache.recalculate_merkle_root(arena, u64_iter(self))?,
            self.len(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test for the `int_log` function.
    #[test]
    fn test_int_log() {
        for i in 0..63 {
            assert_eq!(int_log(2usize.pow(i)), i as usize);
        }
        assert_eq!(int_log(10), 4);
    }
}
