// Author: Zisis Balatsos

// Contents: 
// 1) "criterion" -> used for benchmarking
// 2) "proptest" -> used for generating test data
// 3) "reth_transaction_pool" -> contains the functions blob_tx_priority and fee_delta that are being benchmarked
// 4) "generate_test_data_fee_delta" and "generate_test_data_priority" -> generate random test data using proptest
// 5) "priority_bench" and "fee_jump_bench" -> define the actual benchmark tests
//      (They take a benchmark group, a description, and the input data, then run the respective functions 
//      (blob_tx_priority and fee_delta) within the benchmark.)
// 6) "blob_priority_calculation" -> sets up the benchmark groups, generates the test data, and calls the benchmark functions
// 7) "criterion_group!" and "criterion_main!" macros -> define the entry points for the Criterion benchmarking framework

// Allow missing documentation to avoid warnings for undocumented items
#![allow(missing_docs)]

// Import necessary crates and modules
use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
};
use proptest::{prelude::*, strategy::ValueTree, test_runner::TestRunner};
use reth_transaction_pool::{blob_tx_priority, fee_delta};

// Generate test data for fee delta benchmark
fn generate_test_data_fee_delta() -> (u128, u128) {
    // Use default configuration for Proptest
    let config = ProptestConfig::default();
    // Create a new test runner with the given configuration
    let mut runner = TestRunner::new(config);
    // Generate arbitrary data for (u128, u128) and extract the current value
    prop::arbitrary::any::<(u128, u128)>().new_tree(&mut runner).unwrap().current()
}

// Generate test data for priority benchmark
fn generate_test_data_priority() -> (u128, u128, u128, u128) {
    // Use default configuration for Proptest
    let config = ProptestConfig::default();
    // Create a new test runner with the given configuration
    let mut runner = TestRunner::new(config);
    // Generate arbitrary data for (u128, u128, u128, u128) and extract the current value
    prop::arbitrary::any::<(u128, u128, u128, u128)>().new_tree(&mut runner).unwrap().current()
}

// Benchmark function for priority calculation
fn priority_bench(
    group: &mut BenchmarkGroup<'_, WallTime>,
    description: &str,
    input_data: (u128, u128, u128, u128),
) {
    // Format group identifier with the description
    let group_id = format!("txpool | {description}");

    // Define the benchmark function within the group
    group.bench_function(group_id, |b| {
        b.iter(|| {
            // Call blob_tx_priority with input data, using black_box to prevent compiler optimizations
            black_box(blob_tx_priority(
                black_box(input_data.0),
                black_box(input_data.1),
                black_box(input_data.2),
                black_box(input_data.3),
            ));
        });
    });
}

// Benchmark function for fee delta calculation
fn fee_jump_bench(
    group: &mut BenchmarkGroup<'_, WallTime>,
    description: &str,
    input_data: (u128, u128),
) {
    // Format group identifier with the description
    let group_id = format!("txpool | {description}");

    // Define the benchmark function within the group
    group.bench_function(group_id, |b| {
        b.iter(|| {
            // Call fee_delta with input data, using black_box to prevent compiler optimizations
            black_box(fee_delta(black_box(input_data.0), black_box(input_data.1)));
        });
    });
}

// Main function to setup and execute the benchmarks
fn blob_priority_calculation(c: &mut Criterion) {
    // Create a new benchmark group with a specific name
    let mut group = c.benchmark_group("Blob priority calculation");
    // Generate test data for fee delta benchmark
    let fee_jump_input = generate_test_data_fee_delta();

    // Unstable sorting of unsorted collection
    fee_jump_bench(&mut group, "BenchmarkDynamicFeeJumpCalculation", fee_jump_input);

    let blob_priority_input = generate_test_data_priority();

    // BinaryHeap that is resorted on each update
    priority_bench(&mut group, "BenchmarkPriorityCalculation", blob_priority_input);
}

// Define the criterion group and main function for the benchmark suite
criterion_group!(priority, blob_priority_calculation);
criterion_main!(priority);
