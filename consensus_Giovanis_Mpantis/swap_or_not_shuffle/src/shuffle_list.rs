//! Provides list-shuffling functions matching the Ethereum 2.0 specification.
//!
//! See
//! [compute_shuffled_index](https://github.com/ethereum/eth2.0-specs/blob/v0.12.1/specs/phase0/beacon-chain.md#compute_shuffled_index)
//! for specifications.
//!
//! There are two functions exported by this crate:
//!
//! - `compute_shuffled_index`: given a single index, computes the index resulting from a shuffle.
//! Runs in less time than it takes to run `shuffle_list`.
//! - `shuffle_list`: shuffles an entire list in-place. Runs in less time than it takes to run
//! `compute_shuffled_index` on each index.
//!
//! In general, use `compute_shuffled_index` to calculate the shuffling of a small subset of a much
//! larger list (~250x larger is a good guide, but solid figures yet to be calculated).

mod compute_shuffled_index;
mod shuffle_list;

pub use compute_shuffled_index::compute_shuffled_index;
pub use shuffle_list::shuffle_list;

type Hash256 = ethereum_types::H256;

pub mod compute_shuffled_index {
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
}

pub mod shuffle_list {
    use crate::Hash256;
    use ethereum_hashing::hash_fixed;
    use std::mem;

    const SEED_SIZE: usize = 32;
    const ROUND_SIZE: usize = 1;
    const POSITION_WINDOW_SIZE: usize = 4;
    const PIVOT_VIEW_SIZE: usize = SEED_SIZE + ROUND_SIZE;
    const TOTAL_SIZE: usize = SEED_SIZE + ROUND_SIZE + POSITION_WINDOW_SIZE;

    /// A helper struct to manage the buffer used during shuffling.
    struct Buf([u8; TOTAL_SIZE]);

    impl Buf {
        /// Create a new buffer from the given `seed`.
        ///
        /// ## Panics
        ///
        /// Panics if `seed.len() != 32`.
        fn new(seed: &[u8]) -> Self {
            let mut buf = [0; TOTAL_SIZE];
            buf[0..SEED_SIZE].copy_from_slice(seed);
            Self(buf)
        }

        /// Set the shuffling round.
        fn set_round(&mut self, round: u8) {
            self.0[SEED_SIZE] = round;
        }

        /// Returns the new pivot. It is "raw" because it has not modulo the list size (this must be
        /// done by the caller).
        fn raw_pivot(&self) -> u64 {
            let digest = hash_fixed(&self.0[0..PIVOT_VIEW_SIZE]);

            let mut bytes = [0; mem::size_of::<u64>()];
            bytes[..].copy_from_slice(&digest[0..mem::size_of::<u64>()]);
            u64::from_le_bytes(bytes)
        }

        /// Add the current position into the buffer.
        fn mix_in_position(&mut self, position: usize) {
            self.0[PIVOT_VIEW_SIZE..].copy_from_slice(&position.to_le_bytes()[0..POSITION_WINDOW_SIZE]);
        }

        /// Hash the entire buffer.
        fn hash(&self) -> Hash256 {
            Hash256::from_slice(&hash_fixed(&self.0))
        }
    }

    /// Shuffles an entire list in-place.
    ///
    /// Note: this is equivalent to the `compute_shuffled_index` function, except it shuffles an entire
    /// list not just a single index. With large lists this function has been observed to be 250x
    /// faster than running `compute_shuffled_index` across an entire list.
    ///
    /// Credits to [@protolambda](https://github.com/protolambda) for defining this algorithm.
    ///
    /// Shuffles if `forwards == true`, otherwise un-shuffles.
    /// It holds that: shuffle_list(shuffle_list(l, r, s, true), r, s, false) == l
    ///           and: shuffle_list(shuffle_list(l, r, s, false), r, s, true) == l
    ///
    /// The Eth2.0 spec mostly uses shuffling with `forwards == false`, because backwards
    /// shuffled lists are slightly easier to specify, and slightly easier to compute.
    ///
    /// The forwards shuffling of a list is equivalent to:
    ///
    /// `[indices[x] for i in 0..n, where compute_shuffled_index(x) = i]`
    ///
    /// Whereas the backwards shuffling of a list is:
    ///
    /// `[indices[compute_shuffled_index(i)] for i in 0..n]`
    ///
    /// Returns `None` under any of the following conditions:
    ///  - `list_size == 0`
    ///  - `list_size > 2**24`
    ///  - `list_size > usize::MAX / 2`
    pub fn shuffle_list(
        mut input: Vec<usize>,
        rounds: u8,
        seed: &[u8],
        forwards: bool,
    ) -> Option<Vec<usize>> {
        let list_size = input.len();

        if input.is_empty() || list_size > usize::MAX / 2 || list_size > 2_usize.pow(24) || rounds == 0
        {
            return None;
        }

        let mut buf = Buf::new(seed);

        let mut r = if forwards { 0 } else { rounds - 1 };

        loop {
            buf.set_round(r);

            let pivot = buf.raw_pivot() as usize % list_size;

            let mirror = (pivot + 1) >> 1;

            buf.mix_in_position(pivot >> 8);
            let mut source = buf.hash();
            let mut byte_v = source[(pivot & 0xff) >> 3];

            for i in 0..mirror {
                let j = pivot - i;

                if j & 0xff == 0xff {
                    buf.mix_in_position(j >> 8);
                    source = buf.hash();
                }

                if j & 0x07 == 0x07 {
                    byte_v = source[(j & 0xff) >> 3];
                }
                let bit_v = (byte_v >> (j & 0x07)) & 0x01;

                if bit_v == 1 {
                    input.swap(i, j);
                }
            }

            let mirror = (pivot + list_size + 1) >> 1;
            let end = list_size - 1;

            buf.mix_in_position(end >> 8);
            let mut source = buf.hash();
            let mut byte_v = source[(end & 0xff) >> 3];

            for (loop_iter, i) in ((pivot + 1)..mirror).enumerate() {
                let j = end - loop_iter;

                if j & 0xff == 0xff {
                    buf.mix_in_position(j >> 8);
                    source = buf.hash();
                }

                if j & 0x07 == 0x07 {
                    byte_v = source[(j & 0xff) >> 3];
                }
                let bit_v = (byte_v >> (j & 0x07)) & 0x01;

                if bit_v == 1 {
                    input.swap(i, j);
                }
            }

            if forwards {
                r += 1;
                if r == rounds {
                    break;
                }
            } else {
                if r == 0 {
                    break;
                }
                r -= 1;
            }
        }

        Some(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn returns_none_for_zero_length_list() {
            assert_eq!(None, shuffle_list(vec![], 90, &[42, 42], true));
        }

        #[test]
        #[allow(clippy::assertions_on_constants)]
        fn sanity_check_constants() {
            assert!(TOTAL_SIZE > SEED_SIZE);
            assert!(TOTAL_SIZE > PIVOT_VIEW_SIZE);
            assert!(mem::size_of::<usize>() >= POSITION_WINDOW_SIZE);
        }
    }
}
