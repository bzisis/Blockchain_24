use integer_sqrt::IntegerSquareRoot;
use safe_arith::{ArithError, SafeArith};
use types::*;

/// This type exists to avoid confusing `total_active_balance` with `sqrt_total_active_balance`,
/// since they are used in close proximity and have the same type (`u64`).
#[derive(Copy, Clone)]
pub struct SqrtTotalActiveBalance(u64);

impl SqrtTotalActiveBalance {
    /// Creates a new `SqrtTotalActiveBalance` instance.
    ///
    /// Computes the integer square root of `total_active_balance` and wraps it in `SqrtTotalActiveBalance`.
    ///
    /// # Arguments
    ///
    /// * `total_active_balance` - The total active balance to compute the square root of.
    ///
    /// # Returns
    ///
    /// A new `SqrtTotalActiveBalance` instance.
    pub fn new(total_active_balance: u64) -> Self {
        Self(total_active_balance.integer_sqrt())
    }

    /// Returns the inner value as a `u64`.
    ///
    /// # Returns
    ///
    /// The inner `u64` value representing the square root of the total active balance.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Returns the base reward for some validator.
///
/// Computes the base reward using the formula:
/// `base_reward = validator_effective_balance * spec.base_reward_factor / sqrt_total_active_balance / spec.base_rewards_per_epoch`.
///
/// # Arguments
///
/// * `validator_effective_balance` - The effective balance of the validator.
/// * `sqrt_total_active_balance` - The square root of the total active balance.
/// * `spec` - The chain specification containing parameters for reward calculation.
///
/// # Returns
///
/// The computed base reward as a `Result<u64, ArithError>`.
pub fn get_base_reward(
    validator_effective_balance: u64,
    sqrt_total_active_balance: SqrtTotalActiveBalance,
    spec: &ChainSpec,
) -> Result<u64, ArithError> {
    validator_effective_balance
        .safe_mul(spec.base_reward_factor)?
        .safe_div(sqrt_total_active_balance.as_u64())?
        .safe_div(spec.base_rewards_per_epoch)
}
