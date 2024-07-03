/// Ethereum helper methods
mod ethereum;
pub use ethereum::EthereumHardforks;

/// Optimism helper methods
mod optimism;
pub use optimism::OptimismHardforks;

use crate::{ForkCondition, Hardfork};
use rustc_hash::FxHashMap;

/// Generic trait over a set of ordered hardforks
pub trait Hardforks: Default + Clone {
    /// Retrieves [`ForkCondition`] from `fork`. If `fork` is not present, returns
    /// [`ForkCondition::Never`].
    fn fork<H: Hardfork>(&self, fork: H) -> ForkCondition;

    /// Get an iterator of all hardforks with their respective activation conditions.
    fn forks_iter(&self) -> impl Iterator<Item = (&dyn Hardfork, ForkCondition)>;

    /// Convenience method to check if a fork is active at a given timestamp.
    fn is_fork_active_at_timestamp<H: Hardfork>(&self, fork: H, timestamp: u64) -> bool {
        self.fork(fork).active_at_timestamp(timestamp)
    }

    /// Convenience method to check if a fork is active at a given block number.
    fn is_fork_active_at_block<H: Hardfork>(&self, fork: H, block_number: u64) -> bool {
        self.fork(fork).active_at_block(block_number)
    }
}

/// Ordered list of a chain hardforks that implement [`Hardfork`].
#[derive(Default, Clone, PartialEq, Eq)]
pub struct ChainHardforks {
    forks: Vec<(Box<dyn Hardfork>, ForkCondition)>, /// Vector of hardforks with their conditions
    map: FxHashMap<&'static str, ForkCondition>,    /// HashMap for quick lookup by fork name
}

impl ChainHardforks {
    /// Creates a new [`ChainHardforks`] from a list which **must be ordered** by activation.
    ///
    /// Equivalent Ethereum hardforks **must be included** as well.
    pub fn new(forks: Vec<(Box<dyn Hardfork>, ForkCondition)>) -> Self {
        let map = forks.iter().map(|(fork, condition)| (fork.name(), *condition)).collect();

        Self { forks, map }
    }

    /// Total number of hardforks.
    pub fn len(&self) -> usize {
        self.forks.len()
    }

    /// Checks if the fork list is empty.
    pub fn is_empty(&self) -> bool {
        self.forks.is_empty()
    }

    /// Retrieves [`ForkCondition`] from `fork`. If `fork` is not present, returns
    /// [`ForkCondition::Never`].
    pub fn fork<H: Hardfork>(&self, fork: H) -> ForkCondition {
        self.get(fork).unwrap_or(ForkCondition::Never)
    }

    /// Retrieves [`ForkCondition`] from `fork` if it exists, otherwise `None`.
    pub fn get<H: Hardfork>(&self, fork: H) -> Option<ForkCondition> {
        self.map.get(fork.name()).copied()
    }

    /// Get an iterator of all hardforks with their respective activation conditions.
    pub fn forks_iter(&self) -> impl Iterator<Item = (&dyn Hardfork, ForkCondition)> {
        self.forks.iter().map(|(f, b)| (&**f, *b))
    }

    /// Get last hardfork from the list.
    pub fn last(&self) -> Option<(Box<dyn Hardfork>, ForkCondition)> {
        self.forks.last().map(|(f, b)| (f.clone(), *b))
    }

    /// Convenience method to check if a fork is active at a given timestamp.
    pub fn is_fork_active_at_timestamp<H: Hardfork>(&self, fork: H, timestamp: u64) -> bool {
        self.fork(fork).active_at_timestamp(timestamp)
    }

    /// Convenience method to check if a fork is active at a given block number.
    pub fn is_fork_active_at_block<H: Hardfork>(&self, fork: H, block_number: u64) -> bool {
        self.fork(fork).active_at_block(block_number)
    }

    /// Inserts `fork` into list, updating with a new [`ForkCondition`] if it already exists.
    pub fn insert<H: Hardfork>(&mut self, fork: H, condition: ForkCondition) {
        match self.map.entry(fork.name()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                *entry.get_mut() = condition;
                if let Some((_, inner)) =
                    self.forks.iter_mut().find(|(inner, _)| inner.name() == fork.name())
                {
                    *inner = condition;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(condition);
                self.forks.push((Box::new(fork), condition));
            }
        }
    }

    /// Removes `fork` from list.
    pub fn remove<H: Hardfork>(&mut self, fork: H) {
        self.forks.retain(|(inner_fork, _)| inner_fork.name() != fork.name());
        self.map.remove(fork.name());
    }
}

impl Hardforks for ChainHardforks {
    fn fork<H: Hardfork>(&self, fork: H) -> ForkCondition {
        self.fork(fork)
    }

    fn forks_iter(&self) -> impl Iterator<Item = (&dyn Hardfork, ForkCondition)> {
        self.forks_iter()
    }
}

impl core::fmt::Debug for ChainHardforks {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        /// Debug formatting for ChainHardforks
        f.debug_struct("ChainHardforks")
            .field("0", &self.forks_iter().map(|(hf, cond)| (hf.name(), cond)).collect::<Vec<_>>())
            .finish()
    }
}
