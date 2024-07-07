use std::num::NonZeroUsize;

/// Creates a new `NonZeroUsize` from the given `usize` value.
///
/// # Panics
///
/// Panics if the provided `usize` value is zero.
///
/// # Examples
///
/// ```
/// use std::num::NonZeroUsize;
///
/// fn main() {
///     let n = new_non_zero_usize(5);
///     assert_eq!(n.get(), 5);
/// }
/// ```
pub const fn new_non_zero_usize(x: usize) -> NonZeroUsize {
    match NonZeroUsize::new(x) {
        Some(n) => n,
        None => panic!("Expected a non zero usize."),
    }
}
