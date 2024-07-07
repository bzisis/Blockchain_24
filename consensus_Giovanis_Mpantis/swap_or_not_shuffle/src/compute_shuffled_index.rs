use crate::Hash256;
use ethereum_hashing::{Context, Sha256Context};
use std::cmp::max;

/// Returns `p(index)` in a pseudorandom permutation `p` of `0...list_size-1` with `seed` as entropy.
///
/// Utilizes the 'swap or not' shuffling algorithm found in
/// [this paper](https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf).
/// See the 'generalized domain' algorithm on page 3.
///
/// Note: this function is significantly slower than the `shuffle_list` function in this crate.
/// Using `compute_shuffled_index` to shuffle an entire list, index by index, has been observed to be
/// 250x slower than `shuffle_list`. Therefore, this function is only useful when shuffling a small
/// portion of a much larger list.
///
/// # Parameters
/// - `index`: The index to permute.
/// - `list_size`: The size of the list to permute.
/// - `seed`: A slice of bytes used as the seed for the permutation.
/// - `shuffle_round_count`: The number of rounds to perform in the shuffle.
///
/// # Returns
/// An `Option<usize>` which is `Some(index)` if the shuffle was successful, or `None` under any of
/// the following conditions:
/// - `list_size == 0`
/// - `index >= list_size`
/// - `list_size > 2**24`
/// - `list_size > usize::MAX / 2`
pub fn compute_shuffled_index(
    index: usize,
    list_size: usize,
    seed: &[u8],
    shuffle_round_count: u8,
) -> Option<usize> {
    if list_size == 0
        || index >= list_size
        || list_size > usize::MAX / 2
        || list_size > 2_usize.pow(24)
    {
        return None;
    }

    let mut index = index;
    for round in 0..shuffle_round_count {
        let pivot = bytes_to_int64(&hash_with_round(seed, round)[..]) as usize % list_size;
        index = do_round(seed, index, pivot, round, list_size);
    }
    Some(index)
}

/// Performs a single round of the shuffling algorithm.
///
/// # Parameters
/// - `seed`: A slice of bytes used as the seed for the shuffle.
/// - `index`: The current index in the shuffle.
/// - `pivot`: The pivot index for this round.
/// - `round`: The current round number.
/// - `list_size`: The size of the list being shuffled.
///
/// # Returns
/// The new index after the round is applied.
fn do_round(seed: &[u8], index: usize, pivot: usize, round: u8, list_size: usize) -> usize {
    let flip = (pivot + (list_size - index)) % list_size;
    let position = max(index, flip);
    let source = hash_with_round_and_position(seed, round, position);
    let byte = source[(position % 256) / 8];
    let bit = (byte >> (position % 8)) % 2;
    if bit == 1 {
        flip
    } else {
        index
    }
}

/// Hashes the seed with the given round number and position.
///
/// # Parameters
/// - `seed`: A slice of bytes used as the seed for the hash.
/// - `round`: The current round number.
/// - `position`: The position to include in the hash.
///
/// # Returns
/// A `Hash256` resulting from the hash.
fn hash_with_round_and_position(seed: &[u8], round: u8, position: usize) -> Hash256 {
    let mut context = Context::new();

    context.update(seed);
    context.update(&[round]);
    /*
     * Note: the specification has an implicit assertion in `int_to_bytes4` that `position / 256 <
     * 2**24`. For efficiency, we do not check for that here as it is checked in `compute_shuffled_index`.
     */
    context.update(&(position / 256).to_le_bytes()[0..4]);

    let digest = context.finalize();
    Hash256::from_slice(digest.as_ref())
}

/// Hashes the seed with the given round number.
///
/// # Parameters
/// - `seed`: A slice of bytes used as the seed for the hash.
/// - `round`: The current round number.
///
/// # Returns
/// A `Hash256` resulting from the hash.
fn hash_with_round(seed: &[u8], round: u8) -> Hash256 {
    let mut context = Context::new();

    context.update(seed);
    context.update(&[round]);

    let digest = context.finalize();
    Hash256::from_slice(digest.as_ref())
}

/// Converts a slice of bytes to a `u64`.
///
/// # Parameters
/// - `slice`: A slice of bytes to convert.
///
/// # Returns
/// A `u64` representation of the first 8 bytes of the slice.
fn bytes_to_int64(slice: &[u8]) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&slice[0..8]);
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::H256 as Hash256;

    #[test]
    #[ignore]
    fn fuzz_test() {
        let max_list_size = 2_usize.pow(24);
        let test_runs = 1000;

        // Test at max list_size with the end index.
        for _ in 0..test_runs {
            let index = max_list_size - 1;
            let list_size = max_list_size;
            let seed = Hash256::random();
            let shuffle_rounds = 90;

            assert!(compute_shuffled_index(index, list_size, &seed[..], shuffle_rounds).is_some());
        }

        // Test at max list_size low indices.
        for i in 0..test_runs {
            let index = i;
            let list_size = max_list_size;
            let seed = Hash256::random();
            let shuffle_rounds = 90;

            assert!(compute_shuffled_index(index, list_size, &seed[..], shuffle_rounds).is_some());
        }

        // Test at max list_size high indices.
        for i in 0..test_runs {
            let index = max_list_size - 1 - i;
            let list_size = max_list_size;
            let seed = Hash256::random();
            let shuffle_rounds = 90;

            assert!(compute_shuffled_index(index, list_size, &seed[..], shuffle_rounds).is_some());
        }
    }

    #[test]
    fn returns_none_for_zero_length_list() {
        assert_eq!(None, compute_shuffled_index(100, 0, &[42, 42], 90));
    }

    #[test]
    fn returns_none_for_out_of_bounds_index() {
        assert_eq!(None, compute_shuffled_index(100, 100, &[42, 42], 90));
    }

    #[test]
    fn returns_none_for_too_large_list() {
        assert_eq!(
            None,
            compute_shuffled_index(100, usize::MAX / 2, &[42, 42], 90)
        );
    }
}
