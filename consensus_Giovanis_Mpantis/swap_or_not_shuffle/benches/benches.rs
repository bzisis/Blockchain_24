//! Benchmarking module for shuffle algorithms.
//!
//! This module benchmarks different shuffle algorithms, including single swap and whole list shuffles
//! of various sizes. The benchmarks use the `criterion` crate to measure performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use swap_or_not_shuffle::{compute_shuffled_index, shuffle_list as fast_shuffle};

const SHUFFLE_ROUND_COUNT: u8 = 90;

/// Shuffles a list of the given size using the specified seed and shuffle round count.
///
/// # Arguments
///
/// * `seed` - A slice of bytes used as the seed for the shuffle algorithm.
/// * `list_size` - The size of the list to shuffle.
///
/// # Returns
///
/// A vector of shuffled indices.
fn shuffle_list(seed: &[u8], list_size: usize) -> Vec<usize> {
    let mut output = Vec::with_capacity(list_size);
    for i in 0..list_size {
        output.push(compute_shuffled_index(i, list_size, seed, SHUFFLE_ROUND_COUNT).unwrap());
    }
    output
}

/// Benchmarks various shuffle operations using Criterion.
///
/// This function sets up and runs benchmarks for single index shuffling, 
/// whole list shuffling for small sizes, and whole list shuffling for larger sizes using
/// a different shuffling algorithm.
fn shuffles(c: &mut Criterion) {
    // Benchmark for computing a single shuffled index
    c.bench_function("single swap", move |b| {
        let seed = vec![42; 32];
        b.iter(|| black_box(compute_shuffled_index(0, 10, &seed, SHUFFLE_ROUND_COUNT)))
    });

    // Benchmark for shuffling a whole list of size 8
    c.bench_function("whole list of size 8", move |b| {
        let seed = vec![42; 32];
        b.iter(|| black_box(shuffle_list(&seed, 8)))
    });

    // Benchmarks for shuffling whole lists of varying sizes
    for size in [8, 16, 512, 16_384] {
        c.bench_with_input(
            BenchmarkId::new("whole list shuffle", format!("{size} elements")),
            &size,
            move |b, &n| {
                let seed = vec![42; 32];
                b.iter(|| black_box(shuffle_list(&seed, n)))
            },
        );
    }

    // Benchmark group for larger list shuffles using a faster algorithm
    let mut group = c.benchmark_group("fast");
    group.sample_size(10);
    for size in [512, 16_384, 4_000_000] {
        group.bench_with_input(
            BenchmarkId::new("whole list shuffle", format!("{size} elements")),
            &size,
            move |b, &n| {
                let seed = vec![42; 32];
                let list: Vec<usize> = (0..n).collect();
                b.iter(|| black_box(fast_shuffle(list.clone(), SHUFFLE_ROUND_COUNT, &seed, true)))
            },
        );
    }
    group.finish();
}

// Define the benchmark group and main function for Criterion
criterion_group!(benches, shuffles);
criterion_main!(benches);
