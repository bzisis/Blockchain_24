// Author: Zisis Balatsos

// Contents:
// 1) "criterion" -> used for benchmarking
//    "proptest" -> used for generating test data
//    "reth_transaction_pool::test_utils::MockTransaction" -> provides the mock transactions
// 2) BenchTxPool Trait -> Defines the interface for a transaction pool with methods for adding transactions and reordering based on a base fee
// 3) txpool_reordering Function -> Sets up the benchmark group and iterates over different seed sizes and input sizes to generate test data and run
//      benchmarks using different transaction pool implementations
// 4) xpool_reordering_bench Function -> A generic function that takes a specific transaction pool implementation, initializes it with seed transactions, 
//      and benchmarks the reordering process with additional transactions and base fee adjustments
// 5) generate_test_data Function -> Generates random test data (seed transactions, new transactions, and a base fee) using Proptest
// 6) implementations Module -> Contains different implementations of the transaction pool (VecTxPoolSortStable, VecTxPoolSortUnstable, 
//      and BinaryHeapTxPool), each with their specific methods for adding transactions and reordering them
// 7) Criterion Setup -> criterion_group! and criterion_main! macros define the entry points for the Criterion benchmarking framework

#![allow(missing_docs)]

// Import necessary crates and modules for benchmarking and test data generation
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
};
use proptest::{prelude::*, strategy::ValueTree, test_runner::TestRunner};
use reth_transaction_pool::test_utils::MockTransaction;

// Define a trait for the transaction pool with methods for adding transactions and reordering
// Transaction Pool trait for benchmarking.
trait BenchTxPool: Default {
    fn add_transaction(&mut self, tx: MockTransaction);
    fn reorder(&mut self, base_fee: u64);
}

// Function to benchmark transaction pool reordering
fn txpool_reordering(c: &mut Criterion) {
    // Create a new benchmark group named "Transaction Pool Reordering"
    let mut group = c.benchmark_group("Transaction Pool Reordering");

    // Iterate over different seed sizes and input sizes for the benchmarks
    for seed_size in [1_000, 10_000, 50_000, 100_000] {
        for input_size in [10, 100, 1_000] {
            // Generate test data for the given seed and input sizes
            let (txs, new_txs, base_fee) = generate_test_data(seed_size, input_size);

            // Import the implementations of different transaction pools
            use implementations::*;
            
            // Benchmark using stable sorting of a vector
            // Vanilla sorting of unsorted collection
            txpool_reordering_bench::<VecTxPoolSortStable>(
                &mut group,
                "VecTxPoolSortStable",
                txs.clone(),
                new_txs.clone(),
                base_fee,
            );

            // Benchmark using unstable sorting of a vector
            // Unstable sorting of unsorted collection
            txpool_reordering_bench::<VecTxPoolSortUnstable>(
                &mut group,
                "VecTxPoolSortUnstable",
                txs.clone(),
                new_txs.clone(),
                base_fee,
            );

            // Benchmark using a binary heap
            // BinaryHeap that is resorted on each update
            txpool_reordering_bench::<BinaryHeapTxPool>(
                &mut group,
                "BinaryHeapTxPool",
                txs,
                new_txs,
                base_fee,
            );
        }
    }
}

// Generic function to run a benchmark on a given transaction pool implementation
fn txpool_reordering_bench<T: BenchTxPool>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    description: &str,
    seed: Vec<MockTransaction>,
    new_txs: Vec<MockTransaction>,
    base_fee: u64,
) {
    // Setup function to initialize the transaction pool and add seed transactions
    let setup = || {
        let mut txpool = T::default();
        txpool.reorder(base_fee);

        for tx in &seed {
            txpool.add_transaction(tx.clone());
        }
        (txpool, new_txs.clone())
    };

    // Format a unique identifier for the benchmark
    let group_id = format!(
        "txpool | seed size: {} | input size: {} | {}",
        seed.len(),
        new_txs.len(),
        description
    );

    // Define and run the benchmark function
    group.bench_function(group_id, |b| {
        b.iter_with_setup(setup, |(mut txpool, new_txs)| {
            {
                // Reorder with new base fee (higher)
                let bigger_base_fee = base_fee.saturating_add(10);
                txpool.reorder(bigger_base_fee);

                // Reorder with new base fee after adding transactions (lower)
                for new_tx in new_txs {
                    txpool.add_transaction(new_tx);
                }
                let smaller_base_fee = base_fee.saturating_sub(10);
                txpool.reorder(smaller_base_fee)
            };

            // Prevent compiler optimizations
            std::hint::black_box(());
        });
    });
}

// Function to generate test data for the benchmarks
fn generate_test_data(
    seed_size: usize,
    input_size: usize,
) -> (Vec<MockTransaction>, Vec<MockTransaction>, u64) {

    // Use default configuration for Proptest
    let config = ProptestConfig::default();

    // Create a new test runner with the given configuration
    let mut runner = TestRunner::new(config);

    // Generate a vector of seed transactions
    let txs = prop::collection::vec(any::<MockTransaction>(), seed_size)
        .new_tree(&mut runner)
        .unwrap()
        .current();

    // Generate a vector of new transactions
    let new_txs = prop::collection::vec(any::<MockTransaction>(), input_size)
        .new_tree(&mut runner)
        .unwrap()
        .current();

    // Generate a random base fee
    let base_fee = any::<u64>().new_tree(&mut runner).unwrap().current();

    (txs, new_txs, base_fee)
}

// Module containing different implementations of the transaction pool
mod implementations {
    use super::*;
    use reth_transaction_pool::PoolTransaction;
    use std::collections::BinaryHeap;

    // Implementation of a transaction pool using stable sorting of a vector
    // This implementation appends the transactions and uses [`Vec::sort_by`] function for sorting
    #[derive(Default)]
    pub(crate) struct VecTxPoolSortStable {
        inner: Vec<MockTransaction>,
    }

    impl BenchTxPool for VecTxPoolSortStable {
        fn add_transaction(&mut self, tx: MockTransaction) {
            self.inner.push(tx);
        }

        fn reorder(&mut self, base_fee: u64) {
            self.inner.sort_by(|a, b| {
                a.effective_tip_per_gas(base_fee)
                    .expect("exists")
                    .cmp(&b.effective_tip_per_gas(base_fee).expect("exists"))
            })
        }
    }

    // Implementation of a transaction pool using unstable sorting of a vector
    // This implementation appends the transactions and uses [`Vec::sort_unstable_by`] function for sorting
    #[derive(Default)]
    pub(crate) struct VecTxPoolSortUnstable {
        inner: Vec<MockTransaction>,
    }

    impl BenchTxPool for VecTxPoolSortUnstable {
        fn add_transaction(&mut self, tx: MockTransaction) {
            self.inner.push(tx);
        }

        fn reorder(&mut self, base_fee: u64) {
            self.inner.sort_unstable_by(|a, b| {
                a.effective_tip_per_gas(base_fee)
                    .expect("exists")
                    .cmp(&b.effective_tip_per_gas(base_fee).expect("exists"))
            })
        }
    }

    // Wrapper struct for a transaction with a priority for use in a binary heap
    struct MockTransactionWithPriority {
        tx: MockTransaction,
        priority: u128,
    }

    // Implement equality and ordering traits for the priority struct
    impl PartialEq for MockTransactionWithPriority {
        fn eq(&self, other: &Self) -> bool {
            self.priority == other.priority
        }
    }

    impl Eq for MockTransactionWithPriority {}

    impl PartialOrd for MockTransactionWithPriority {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for MockTransactionWithPriority {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.priority.cmp(&other.priority)
        }
    }

    // Implementation of a transaction pool using a binary heap
    // This implementation uses `BinaryHeap` which is drained and reconstructed on each reordering
    #[derive(Default)]
    pub(crate) struct BinaryHeapTxPool {
        inner: BinaryHeap<MockTransactionWithPriority>,
        base_fee: Option<u64>,
    }

    impl BenchTxPool for BinaryHeapTxPool {
        fn add_transaction(&mut self, tx: MockTransaction) {
            let priority = self
                .base_fee
                .as_ref()
                .map(|bf| tx.effective_tip_per_gas(*bf).expect("set"))
                .unwrap_or_default();
            self.inner.push(MockTransactionWithPriority { tx, priority });
        }

        fn reorder(&mut self, base_fee: u64) {
            self.base_fee = Some(base_fee);

            // Drain and reconstruct the binary heap with updated priorities
            let drained = self.inner.drain();
            self.inner = BinaryHeap::from_iter(drained.map(|mock| {
                let priority = mock.tx.effective_tip_per_gas(base_fee).expect("set");
                MockTransactionWithPriority { tx: mock.tx, priority }
            }));
        }
    }
}

// Define the criterion group and main function for the benchmark suite
criterion_group!(reorder, txpool_reordering);
criterion_main!(reorder);
