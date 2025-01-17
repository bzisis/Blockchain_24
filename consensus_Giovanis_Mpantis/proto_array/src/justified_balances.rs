use safe_arith::{ArithError, SafeArith};
use types::{BeaconState, EthSpec};

/// Holds balances and related information for validators in a justified state.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct JustifiedBalances {
    /// The effective balances for every validator in a given justified state.
    ///
    /// Any validator who is not active in the epoch of the justified state is assigned a balance of
    /// zero.
    pub effective_balances: Vec<u64>,
    /// The sum of `self.effective_balances`.
    pub total_effective_balance: u64,
    /// The number of active validators included in `self.effective_balances`.
    pub num_active_validators: u64,
}

impl JustifiedBalances {
    /// Constructs `JustifiedBalances` from a given justified state.
    ///
    /// # Arguments
    ///
    /// * `state` - The beacon state from which to derive justified balances.
    ///
    /// # Errors
    ///
    /// Returns an `ArithError` if there's an arithmetic overflow during balance computation.
    pub fn from_justified_state<E: EthSpec>(state: &BeaconState<E>) -> Result<Self, ArithError> {
        let current_epoch = state.current_epoch();
        let mut total_effective_balance = 0u64;
        let mut num_active_validators = 0u64;

        let effective_balances = state
            .validators()
            .iter()
            .map(|validator| {
                if !validator.slashed && validator.is_active_at(current_epoch) {
                    total_effective_balance.safe_add_assign(validator.effective_balance)?;
                    num_active_validators.safe_add_assign(1)?;

                    Ok(validator.effective_balance)
                } else {
                    Ok(0)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            effective_balances,
            total_effective_balance,
            num_active_validators,
        })
    }

    /// Constructs `JustifiedBalances` from a given vector of effective balances.
    ///
    /// # Arguments
    ///
    /// * `effective_balances` - A vector containing effective balances of validators.
    ///
    /// # Errors
    ///
    /// Returns an `ArithError` if there's an arithmetic overflow during balance computation.
    pub fn from_effective_balances(effective_balances: Vec<u64>) -> Result<Self, ArithError> {
        let mut total_effective_balance = 0;
        let mut num_active_validators = 0;

        for &balance in &effective_balances {
            if balance != 0 {
                total_effective_balance.safe_add_assign(balance)?;
                num_active_validators.safe_add_assign(1)?;
            }
        }

        Ok(Self {
            effective_balances,
            total_effective_balance,
            num_active_validators,
        })
    }
}
