// Allow certain lints for this file
#![allow(missing_docs, unreachable_pub)]

// Import necessary crates
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion}; // For benchmarking
use proptest::{prelude::*, strategy::ValueTree, test_runner::TestRunner}; // For property-based testing and generating test data
use proptest_arbitrary_interop::arb; // Interop between proptest and custom types
use rayon::ThreadPoolBuilder; // For building a thread pool
use reth_primitives::{Account, B256, U256}; // Custom primitives likely related to Ethereum
use reth_provider::{
    bundle_state::HashedStateChanges, providers::ConsistentDbView,
    test_utils::create_test_provider_factory,
}; // Custom provider for database operations
use reth_tasks::pool::BlockingTaskPool; // Custom task pool
use reth_trie::{
    hashed_cursor::HashedPostStateCursorFactory, HashedPostState, HashedStorage, StateRoot,
}; // Custom trie structures
use reth_trie_parallel::{async_root::AsyncStateRoot, parallel_root::ParallelStateRoot}; // Parallel and async trie root calculations
use std::collections::HashMap; // For using HashMap

// Main function for benchmarking state root calculation
pub fn calculate_state_root(c: &mut Criterion) {
    // Create a benchmark group named "Calculate State Root" and set sample size to 20
    let mut group = c.benchmark_group("Calculate State Root");
    group.sample_size(20);

    // Create a new Tokio runtime for async operations
    let runtime = tokio::runtime::Runtime::new().unwrap();
    // Create a blocking task pool using Rayon
    let blocking_pool = BlockingTaskPool::new(ThreadPoolBuilder::default().build().unwrap());

    // Define different sizes of test data
    for size in [1_000, 3_000, 5_000, 10_000] {
        // Generate test data of the given size
        let (db_state, updated_state) = generate_test_data(size);
        let provider_factory = create_test_provider_factory();

        // Setup the initial state in the database
        {
            let provider_rw = provider_factory.provider_rw().unwrap();
            HashedStateChanges(db_state).write_to_db(provider_rw.tx_ref()).unwrap();
            let (_, updates) =
                StateRoot::from_tx(provider_rw.tx_ref()).root_with_updates().unwrap();
            updates.write_to_database(provider_rw.tx_ref()).unwrap();
            provider_rw.commit().unwrap();
        }

        // Create a consistent view of the database
        let view = ConsistentDbView::new(provider_factory.clone(), None);

        // Benchmark synchronous state root calculation
        group.bench_function(BenchmarkId::new("sync root", size), |b| {
            b.to_async(&runtime).iter_with_setup(
                || {
                    let sorted_state = updated_state.clone().into_sorted();
                    let prefix_sets = updated_state.construct_prefix_sets().freeze();
                    let provider = provider_factory.provider().unwrap();
                    (provider, sorted_state, prefix_sets)
                },
                |(provider, sorted_state, prefix_sets)| async move {
                    StateRoot::from_tx(provider.tx_ref())
                        .with_hashed_cursor_factory(HashedPostStateCursorFactory::new(
                            provider.tx_ref(),
                            &sorted_state,
                        ))
                        .with_prefix_sets(prefix_sets)
                        .root()
                },
            )
        });

        // Benchmark parallel state root calculation
        group.bench_function(BenchmarkId::new("parallel root", size), |b| {
            b.to_async(&runtime).iter_with_setup(
                || ParallelStateRoot::new(view.clone(), updated_state.clone()),
                |calculator| async { calculator.incremental_root() },
            );
        });

        // Benchmark asynchronous state root calculation
        group.bench_function(BenchmarkId::new("async root", size), |b| {
            b.to_async(&runtime).iter_with_setup(
                || AsyncStateRoot::new(view.clone(), blocking_pool.clone(), updated_state.clone()),
                |calculator| calculator.incremental_root(),
            );
        });
    }
}

// Function to generate test data
fn generate_test_data(size: usize) -> (HashedPostState, HashedPostState) {
    let storage_size = 1_000; // Set storage size
    let mut runner = TestRunner::new(ProptestConfig::default()); // Create a Proptest runner

    use proptest::{collection::hash_map, sample::subsequence};
    
    // Generate initial database state with random data
    let db_state = hash_map(
        any::<B256>(),
        (
            arb::<Account>().prop_filter("non empty account", |a| !a.is_empty()),
            hash_map(
                any::<B256>(),
                any::<U256>().prop_filter("non zero value", |v| !v.is_zero()),
                storage_size,
            ),
        ),
        size,
    )
    .new_tree(&mut runner)
    .unwrap()
    .current();

    // Select keys to update
    let keys = db_state.keys().cloned().collect::<Vec<_>>();
    let keys_to_update = subsequence(keys, size / 2).new_tree(&mut runner).unwrap().current();

    // Generate updated storages
    let updated_storages = keys_to_update
        .into_iter()
        .map(|address| {
            let (_, storage) = db_state.get(&address).unwrap();
            let slots = storage.keys().cloned().collect::<Vec<_>>();
            let slots_to_update =
                subsequence(slots, storage_size / 2).new_tree(&mut runner).unwrap().current();
            (
                address,
                slots_to_update
                    .into_iter()
                    .map(|slot| (slot, any::<U256>().new_tree(&mut runner).unwrap().current()))
                    .collect::<HashMap<_, _>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    // Return the generated initial and updated states
    (
        HashedPostState::default()
            .with_accounts(
                db_state.iter().map(|(address, (account, _))| (*address, Some(*account))),
            )
            .with_storages(db_state.into_iter().map(|(address, (_, storage))| {
                (address, HashedStorage::from_iter(false, storage))
            })),
        HashedPostState::default().with_storages(
            updated_storages
                .into_iter()
                .map(|(address, storage)| (address, HashedStorage::from_iter(false, storage))),
        ),
    )
}

// Register the benchmark function
criterion_group!(state_root, calculate_state_root);
// Define the main entry point for the Criterion benchmark
criterion_main!(state_root);
