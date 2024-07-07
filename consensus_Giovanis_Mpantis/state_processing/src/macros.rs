/// Macro for verifying a condition and returning an error if the condition is false.
///
/// # Arguments
///
/// * `$condition`: The condition to verify.
/// * `$result`: The result to return if the condition is false.
///
/// # Example
///
/// ```rust
/// # use crate::per_block_processing::errors::BlockOperationError;
/// # macro_rules! verify {
/// #     ($condition: expr, $result: expr) => {
/// #         if !$condition {
/// #             return Err(BlockOperationError::invalid($result));
/// #         }
/// #     };
/// # }
/// #
/// let x = 5;
/// verify!(x == 5, "x must be equal to 5");
/// ```
macro_rules! verify {
    ($condition: expr, $result: expr) => {
        if !$condition {
            return Err(crate::per_block_processing::errors::BlockOperationError::invalid($result));
        }
    };
}

/// Macro for verifying a condition and returning a specified error.
///
/// # Arguments
///
/// * `$condition`: The condition to verify.
/// * `$result`: The error value to return if the condition is false.
///
/// # Example
///
/// ```rust
/// # macro_rules! block_verify {
/// #     ($condition: expr, $result: expr) => {
/// #         if !$condition {
/// #             return Err($result);
/// #         }
/// #     };
/// # }
/// #
/// let y = 10;
/// block_verify!(y > 5, "y must be greater than 5");
/// ```
macro_rules! block_verify {
    ($condition: expr, $result: expr) => {
        if !$condition {
            return Err($result);
        }
    };
}
