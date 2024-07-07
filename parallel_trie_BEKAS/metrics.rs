// Importing necessary modules and types
use crate::stats::ParallelTrieStats; // For parallel trie statistics
use metrics::Histogram; // For histogram metrics
use reth_metrics::Metrics; // For deriving Metrics trait
use reth_trie::metrics::{TrieRootMetrics, TrieType}; // For trie root metrics and types

/// Structure for storing metrics related to parallel state root computations.
#[derive(Debug)] // Automatically generate the Debug trait for the struct
pub struct ParallelStateRootMetrics {
    /// Metrics for the state trie.
    pub state_trie: TrieRootMetrics,
    /// Metrics for parallel trie computations.
    pub parallel: ParallelTrieMetrics,
    /// Metrics for the storage trie.
    pub storage_trie: TrieRootMetrics,
}

// Implementation of the Default trait for ParallelStateRootMetrics
impl Default for ParallelStateRootMetrics {
    fn default() -> Self {
        Self {
            // Initialize state_trie with new TrieRootMetrics of type State
            state_trie: TrieRootMetrics::new(TrieType::State),
            // Initialize parallel with default ParallelTrieMetrics
            parallel: ParallelTrieMetrics::default(),
            // Initialize storage_trie with new TrieRootMetrics of type Storage
            storage_trie: TrieRootMetrics::new(TrieType::Storage),
        }
    }
}

// Implementation block for ParallelStateRootMetrics
impl ParallelStateRootMetrics {
    /// Method to record state trie metrics
    pub fn record_state_trie(&self, stats: ParallelTrieStats) {
        // Record statistics related to the state trie
        self.state_trie.record(stats.trie_stats());
        // Record the number of precomputed storage roots
        self.parallel.precomputed_storage_roots.record(stats.precomputed_storage_roots() as f64);
        // Record the number of missed leaves
        self.parallel.missed_leaves.record(stats.missed_leaves() as f64);
    }
}

/// Metrics specifically for parallel trie computations.
#[derive(Metrics)] // Derive the Metrics trait for automatic metric handling
#[metrics(scope = "trie_parallel")] // Specify the scope for these metrics
pub struct ParallelTrieMetrics {
    /// Histogram for the number of storage roots computed in parallel.
    pub precomputed_storage_roots: Histogram,
    /// Histogram for the number of leaves that missed pre-computation of storage roots.
    pub missed_leaves: Histogram,
}
