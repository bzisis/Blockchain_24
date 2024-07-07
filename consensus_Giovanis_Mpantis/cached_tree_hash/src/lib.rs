//! This module provides functionalities for caching, tree hash calculation, and related utilities.

mod cache;
mod cache_arena;
mod impls;
#[cfg(test)]
mod test;

use smallvec::SmallVec;

/// A type alias for a small vector with a capacity of 8.
type SmallVec8<T> = SmallVec<[T; 8]>;

/// A type alias for the `CacheArena` specialized for `Hash256`.
pub type CacheArena = cache_arena::CacheArena<Hash256>;

pub use crate::cache::TreeHashCache;
pub use crate::impls::int_log;
use ethereum_types::H256 as Hash256;

/// An enumeration of possible errors that can occur while working with the cache and tree hashing.
#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    /// Attempting to provide more than 2^depth leaves to a Merkle tree is disallowed.
    TooManyLeaves,
    /// Shrinking a Merkle tree cache by providing it with fewer leaves than it currently has is
    /// disallowed (for simplicity).
    CannotShrink,
    /// The cache is inconsistent with the list of dirty indices provided.
    CacheInconsistent,
    /// An error occurred within the `CacheArena`.
    CacheArenaError(cache_arena::Error),
    /// Unable to find the left index in the Merkle tree.
    MissingLeftIdx(usize),
}

impl From<cache_arena::Error> for Error {
    /// Converts a `cache_arena::Error` into an `Error`.
    fn from(e: cache_arena::Error) -> Error {
        Error::CacheArenaError(e)
    }
}

/// Trait for types which can make use of a cache to accelerate the calculation of their tree hash root.
pub trait CachedTreeHash<Cache> {
    /// Create a new cache appropriate for use with values of this type.
    ///
    /// # Arguments
    ///
    /// * `arena` - A mutable reference to the `CacheArena` where the cache will be created.
    fn new_tree_hash_cache(&self, arena: &mut CacheArena) -> Cache;

    /// Update the cache and use it to compute the tree hash root for `self`.
    ///
    /// # Arguments
    ///
    /// * `arena` - A mutable reference to the `CacheArena`.
    /// * `cache` - A mutable reference to the cache to be updated and used.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the cache update or tree hash root calculation fails.
    fn recalculate_tree_hash_root(
        &self,
        arena: &mut CacheArena,
        cache: &mut Cache,
    ) -> Result<Hash256, Error>;
}
