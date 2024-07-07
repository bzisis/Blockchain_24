use crate::{Result, SafeArith};

/// Extension trait for iterators, providing a safe replacement for `sum`.
///
/// This trait adds the `safe_sum` method to any iterator, which performs a
/// sum operation that returns a `Result` to handle potential arithmetic
/// overflows.
///
/// # Examples
///
/// ```
/// use crate::SafeArithIter;
/// let v = vec![1, 2, 3];
/// assert_eq!(v.into_iter().safe_sum(), Ok(6));
/// ```
pub trait SafeArithIter<T> {
    /// Sums the elements of the iterator, returning a `Result` to handle
    /// potential overflows.
    ///
    /// # Errors
    ///
    /// Returns an error if the sum exceeds the bounds of the type `T`.
    fn safe_sum(self) -> Result<T>;
}

impl<I, T> SafeArithIter<T> for I
where
    I: Iterator<Item = T> + Sized,
    T: SafeArith,
{
    fn safe_sum(mut self) -> Result<T> {
        self.try_fold(T::ZERO, |acc, x| acc.safe_add(x))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ArithError;

    /// Tests the safe_sum method with an empty vector.
    #[test]
    fn empty_sum() {
        let v: Vec<u64> = vec![];
        assert_eq!(v.into_iter().safe_sum(), Ok(0));
    }

    /// Tests the safe_sum method with a small vector of unsigned integers.
    #[test]
    fn unsigned_sum_small() {
        let arr = [400u64, 401, 402, 403, 404, 405, 406];
        assert_eq!(
            arr.iter().copied().safe_sum().unwrap(),
            arr.iter().copied().sum()
        );
    }

    /// Tests the safe_sum method with a vector that causes an overflow.
    #[test]
    fn unsigned_sum_overflow() {
        let v = vec![u64::MAX, 1];
        assert_eq!(v.into_iter().safe_sum(), Err(ArithError::Overflow));
    }

    /// Tests the safe_sum method with a small vector of signed integers.
    #[test]
    fn signed_sum_small() {
        let v = vec![-1i64, -2i64, -3i64, 3, 2, 1];
        assert_eq!(v.into_iter().safe_sum(), Ok(0));
    }

    /// Tests the safe_sum method with a vector that causes an overflow above the maximum value.
    #[test]
    fn signed_sum_overflow_above() {
        let v = vec![1, 2, 3, 4, i16::MAX, 0, 1, 2, 3];
        assert_eq!(v.into_iter().safe_sum(), Err(ArithError::Overflow));
    }

    /// Tests the safe_sum method with a vector that causes an overflow below the minimum value.
    #[test]
    fn signed_sum_overflow_below() {
        let v = vec![i16::MIN, -1];
        assert_eq!(v.into_iter().safe_sum(), Err(ArithError::Overflow));
    }

    /// Tests the safe_sum method with a vector that almost causes an overflow.
    #[test]
    fn signed_sum_almost_overflow() {
        let arr = [i64::MIN, 1, -1i64, i64::MAX, i64::MAX, 1];
        assert_eq!(
            arr.iter().copied().safe_sum().unwrap(),
            arr.iter().copied().sum()
        );
    }
}
