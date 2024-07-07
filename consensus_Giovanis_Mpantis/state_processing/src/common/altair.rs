use integer_sqrt::IntegerSquareRoot;
use safe_arith::{ArithError, SafeArith};
use types::*;

/// This type exists to avoid confusing `total_active_balance` with `base_reward_per_increment`,
/// since they are used in close proximity and are of the same type (`u64`).
#[derive(Copy, Clone)]
pub struct BaseRewardPerIncrement(u64);

impl BaseRewardPerIncrement {
    /// Creates a new `BaseRewardPerIncrement` from the `total_active_balance` and `ChainSpec`.
    ///
    /// # Arguments
    ///
    /// * `total_active_balance` - The total active balance.
    /// * `spec` - A reference to the `ChainSpec`.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the new `BaseRewardPerIncrement` or an `ArithError`.
    pub fn new(total_active_balance: u64, spec: &ChainSpec) -> Result<Self, ArithError> {
        get_base_reward_per_increment(total_active_balance, spec).map(Self)
    }

    /// Returns the inner `u64` value of the `BaseRewardPerIncrement`.
    ///
    /// # Returns
    ///
    /// The inner `u64` value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Returns the base reward for a validator.
///
/// This function has a different interface to the specification since it accepts the
/// `base_reward_per_increment` without computing it each time. Avoiding the re-computation has
/// shown to be a significant optimization.
///
/// Spec v1.1.0
///
/// # Arguments
///
/// * `validator_effective_balance` - The effective balance of the validator.
/// * `base_reward_per_increment` - The base reward per increment.
/// * `spec` - A reference to the `ChainSpec`.
///
/// # Returns
///
/// A `Result` containing either the base reward or an `Error`.
pub fn get_base_reward(
    validator_effective_balance: u64,
    base_reward_per_increment: BaseRewardPerIncrement,
    spec: &ChainSpec,
) -> Result<u64, Error> {
    validator_effective_balance
        .safe_div(spec.effective_balance_increment)?
        .safe_mul(base_reward_per_increment.as_u64())
        .map_err(Into::into)
}

/// Computes the base reward per increment.
///
/// Spec v1.1.0
///
/// # Arguments
///
/// * `total_active_balance` - The total active balance.
/// * `spec` - A reference to the `ChainSpec`.
///
/// # Returns
///
/// A `Result` containing either the base reward per increment or an `ArithError`.
fn get_base_reward_per_increment(
    total_active_balance: u64,
    spec: &ChainSpec,
) -> Result<u64, ArithError> {
    spec.effective_balance_increment
        .safe_mul(spec.base_reward_factor)?
        .safe_div(total_active_balance.integer_sqrt())
}
