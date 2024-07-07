use crate::{
    common::update_progressive_balances_cache::initialize_progressive_balances_cache,
    epoch_cache::{initialize_epoch_cache, PreEpochCache},
    per_epoch_processing::{Delta, Error, ParticipationEpochSummary},
};
use itertools::izip;
use safe_arith::{SafeArith, SafeArithIter};
use std::cmp::{max, min};
use std::collections::BTreeSet;
use types::{
    consts::altair::{
        NUM_FLAG_INDICES, PARTICIPATION_FLAG_WEIGHTS, TIMELY_HEAD_FLAG_INDEX,
        TIMELY_TARGET_FLAG_INDEX, WEIGHT_DENOMINATOR,
    },
    milhouse::Cow,
    ActivationQueue, BeaconState, BeaconStateError, ChainSpec, Epoch, EthSpec, ExitCache, ForkName,
    ParticipationFlags, ProgressiveBalancesCache, RelativeEpoch, Unsigned, Validator,
};

/// Configuration for processing a single epoch pass.
pub struct SinglePassConfig {
    /// Enable inactivity updates.
    pub inactivity_updates: bool,
    /// Enable rewards and penalties.
    pub rewards_and_penalties: bool,
    /// Enable registry updates.
    pub registry_updates: bool,
    /// Enable slashings processing.
    pub slashings: bool,
    /// Enable effective balance updates.
    pub effective_balance_updates: bool,
}

impl Default for SinglePassConfig {
    /// Creates a default `SinglePassConfig` with all options enabled.
    fn default() -> SinglePassConfig {
        Self::enable_all()
    }
}

impl SinglePassConfig {
    /// Creates a `SinglePassConfig` with all options enabled.
    pub fn enable_all() -> SinglePassConfig {
        Self {
            inactivity_updates: true,
            rewards_and_penalties: true,
            registry_updates: true,
            slashings: true,
            effective_balance_updates: true,
        }
    }

    /// Creates a `SinglePassConfig` with all options disabled.
    pub fn disable_all() -> SinglePassConfig {
        SinglePassConfig {
            inactivity_updates: false,
            rewards_and_penalties: false,
            registry_updates: false,
            slashings: false,
            effective_balance_updates: false,
        }
    }
}

/// Values from the state that are immutable throughout epoch processing.
struct StateContext {
    current_epoch: Epoch,
    next_epoch: Epoch,
    is_in_inactivity_leak: bool,
    total_active_balance: u64,
    churn_limit: u64,
    fork_name: ForkName,
}

/// Context for calculating rewards and penalties.
struct RewardsAndPenaltiesContext {
    unslashed_participating_increments_array: [u64; NUM_FLAG_INDICES],
    active_increments: u64,
}

/// Context for slashings processing.
struct SlashingsContext {
    adjusted_total_slashing_balance: u64,
    target_withdrawable_epoch: Epoch,
}

/// Context for effective balances processing.
struct EffectiveBalancesContext {
    downward_threshold: u64,
    upward_threshold: u64,
}

/// Information about a validator used during epoch processing.
#[derive(Debug, PartialEq, Clone)]
pub struct ValidatorInfo {
    pub index: usize,
    pub effective_balance: u64,
    pub base_reward: u64,
    pub is_eligible: bool,
    pub is_slashed: bool,
    pub is_active_current_epoch: bool,
    pub is_active_previous_epoch: bool,
    /// Used for determining rewards.
    pub previous_epoch_participation: ParticipationFlags,
    /// Used for updating the progressive balances cache for next epoch.
    pub current_epoch_participation: ParticipationFlags,
}

impl ValidatorInfo {
    /// Checks if a validator is unslashed and has a particular flag set.
    #[inline]
    pub fn is_unslashed_participating_index(&self, flag_index: usize) -> Result<bool, Error> {
        Ok(self.is_active_previous_epoch
            && !self.is_slashed
            && self
                .previous_epoch_participation
                .has_flag(flag_index)
                .map_err(|_| Error::InvalidFlagIndex(flag_index))?)
    }
}

/// Processes a single epoch in one pass.
///
/// This function updates the given `BeaconState` according to the configuration provided
/// in `SinglePassConfig`.
///
/// # Arguments
///
/// * `state` - The mutable reference to the `BeaconState`.
/// * `spec` - The chain specification.
/// * `conf` - The configuration for processing the epoch.
///
/// # Returns
///
/// * `Result<ParticipationEpochSummary<E>, Error>` - The summary of the participation epoch.
pub fn process_epoch_single_pass<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
    conf: SinglePassConfig,
) -> Result<ParticipationEpochSummary<E>, Error> {
    initialize_epoch_cache(state, spec)?;
    initialize_progressive_balances_cache(state, spec)?;
    state.build_exit_cache(spec)?;
    state.build_committee_cache(RelativeEpoch::Previous, spec)?;
    state.build_committee_cache(RelativeEpoch::Current, spec)?;

    let previous_epoch = state.previous_epoch();
    let current_epoch = state.current_epoch();
    let next_epoch = state.next_epoch()?;
    let is_in_inactivity_leak = state.is_in_inactivity_leak(previous_epoch, spec)?;
    let total_active_balance = state.get_total_active_balance()?;
    let churn_limit = state.get_validator_churn_limit(spec)?;
    let activation_churn_limit = state.get_activation_churn_limit(spec)?;
    let finalized_checkpoint = state.finalized_checkpoint();
    let fork_name = state.fork_name_unchecked();

    let state_ctxt = &StateContext {
        current_epoch,
        next_epoch,
        is_in_inactivity_leak,
        total_active_balance,
        churn_limit,
        fork_name,
    };

    let slashings_ctxt = &SlashingsContext::new(state, state_ctxt, spec)?;
    let mut next_epoch_cache = PreEpochCache::new_for_next_epoch(state)?;

    let (
        validators,
        balances,
        previous_epoch_participation,
        current_epoch_participation,
        inactivity_scores,
        progressive_balances,
        exit_cache,
        epoch_cache,
    ) = state.mutable_validator_fields()?;

    let num_validators = validators.len();

    let summary = ParticipationEpochSummary::new(
        validators.clone(),
        previous_epoch_participation.clone(),
        current_epoch_participation.clone(),
        previous_epoch,
        current_epoch,
    );

    let rewards_ctxt = &RewardsAndPenaltiesContext::new(progressive_balances, state_ctxt, spec)?;
    let activation_queue = &epoch_cache
        .activation_queue()?
        .get_validators_eligible_for_activation(
            finalized_checkpoint.epoch,
            activation_churn_limit as usize,
        );
    let effective_balances_ctxt = &EffectiveBalancesContext::new(spec)?;

    let mut validators_iter = validators.iter_cow();
    let mut balances_iter = balances.iter_cow();
    let mut inactivity_scores_iter = inactivity_scores.iter_cow();

    let mut next_epoch_total_active_balance = 0;
    let mut next_epoch_activation_queue = ActivationQueue::default();

    for (index, &previous_epoch_participation, &current_epoch_participation) in izip!(
        0..num_validators,
        previous_epoch_participation.iter(),
        current_epoch_participation.iter(),
    ) {
        let (_, mut validator) = validators_iter
            .next_cow()
            .ok_or(BeaconStateError::UnknownValidator(index))?;
        let (_, mut balance) = balances_iter
            .next_cow()
            .ok_or(BeaconStateError::UnknownValidator(index))?;
        let (_, mut inactivity_score) = inactivity_scores_iter
            .next_cow()
            .ok_or(BeaconStateError::UnknownValidator(index))?;

        let is_active_current_epoch = validator.is_active_at(current_epoch);
        let is_active_previous_epoch = validator.is_active_at(previous_epoch);
        let is_eligible = is_active_previous_epoch
            || (validator.slashed && previous_epoch.safe_add(1)? < validator.withdrawable_epoch);

        let base_reward = if is_eligible {
            epoch_cache.get_base_reward(index)?
        } else {
            0
        };

        let validator_info = &ValidatorInfo {
            index,
            effective_balance: validator.effective_balance,
            base_reward,
            is_eligible,
            is_slashed: validator.slashed,
            is_active_current_epoch,
            is_active_previous_epoch,
            previous_epoch_participation,
            current_epoch_participation,
        };

        if current_epoch != E::genesis_epoch() {
            if conf.inactivity_updates {
                process_single_inactivity_update(
                    &mut inactivity_score,
                    validator_info,
                    state_ctxt,
                    spec,
                )?;
            }

            if conf.rewards_and_penalties {
                process_single_reward_and_penalty(
                    &mut balance,
                    &inactivity_score,
                    validator_info,
                    rewards_ctxt,
                    state_ctxt,
                    spec,
                )?;
            }
        }

        if conf.registry_updates {
            process_single_registry_update(
                &mut validator,
                validator_info,
                exit_cache,
                activation_queue,
                &mut next_epoch_activation_queue,
                state_ctxt,
                spec,
            )?;
        }

        if conf.slashings {
            process_single_slashing(&mut balance, &validator, slashings_ctxt, state_ctxt, spec)?;
        }

        if conf.effective_balance_updates {
            process_single_effective_balance_update(
                *balance,
                &mut validator,
                validator_info,
                &mut next_epoch_total_active_balance,
                &mut next_epoch_cache,
                progressive_balances,
                effective_balances_ctxt,
                state_ctxt,
                spec,
            )?;
        }
    }

    if conf.effective_balance_updates {
        state.set_total_active_balance(next_epoch, next_epoch_total_active_balance)?;
    }

    state.update_next_epoch_cache(next_epoch_cache)?;
    progressive_balances.update_validators(validators)?;

    Ok(summary)
}

fn process_single_reward_and_penalty(
    balance: &mut Cow<u64>,
    inactivity_score: &u64,
    info: &ValidatorInfo,
    rewards_ctxt: &RewardsAndPenaltiesContext,
    state_ctxt: &StateContext,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let mut delta = Delta::default();

    for flag_index in 0..NUM_FLAG_INDICES {
        let base_reward_per_flag = info
            .base_reward
            .safe_mul(PARTICIPATION_FLAG_WEIGHTS[flag_index])?
            .safe_div(WEIGHT_DENOMINATOR)?;
        let base_reward_per_flag = min(base_reward_per_flag, info.effective_balance);
        let reward = base_reward_per_flag
            .safe_mul(rewards_ctxt.unslashed_participating_increments_array[flag_index])?
            .safe_div(rewards_ctxt.active_increments)?;
        delta.increment(reward)?;

        if !info.is_unslashed_participating_index(flag_index)? {
            let penalty = base_reward_per_flag;
            delta.decrement(penalty)?;
        }
    }

    let target_inactivity_penalty = if !info.is_unslashed_participating_index(TIMELY_TARGET_FLAG_INDEX)? {
        let target_inactivity_penalty = info.effective_balance
            .safe_mul(state_ctxt.current_epoch.safe_sub(info.index)?)?
            .safe_div(spec.inactivity_penalty_quotient)?;

        let mut penalty = delta.decrement(target_inactivity_penalty)?;
        penalty.safe_add(base_reward_per_flag)?
    } else {
        delta
    };

    let head_inactivity_penalty = if !info.is_unslashed_participating_index(TIMELY_HEAD_FLAG_INDEX)? {
        let head_inactivity_penalty = info.effective_balance
            .safe_mul(state_ctxt.current_epoch.safe_sub(info.index)?)?
            .safe_div(spec.inactivity_penalty_quotient)?;
        
        let mut penalty = target_inactivity_penalty.decrement(head_inactivity_penalty)?;
        penalty.safe_add(base_reward_per_flag)?
    } else {
        target_inactivity_penalty
    };

    *balance += delta.balance();
    Ok(())
}

/// Updates the inactivity score for a single validator.
fn process_single_inactivity_update(
    inactivity_score: &mut Cow<u64>,
    info: &ValidatorInfo,
    state_ctxt: &StateContext,
    spec: &ChainSpec,
) -> Result<(), Error> {
    if info.is_eligible && !info.is_unslashed_participating_index(TIMELY_TARGET_FLAG_INDEX)? {
        let score_increase = if state_ctxt.is_in_inactivity_leak {
            2
        } else {
            1
        };

        inactivity_score.safe_add_assign(score_increase)?;
    } else {
        inactivity_score.safe_sub_assign(min(*inactivity_score, 1))?;
    }

    Ok(())
}

/// Updates the registry for a single validator.
fn process_single_registry_update(
    validator: &mut Cow<Validator>,
    info: &ValidatorInfo,
    exit_cache: &ExitCache,
    activation_queue: &ActivationQueue,
    next_epoch_activation_queue: &mut ActivationQueue,
    state_ctxt: &StateContext,
    spec: &ChainSpec,
) -> Result<(), Error> {
    if validator.slashed && state_ctxt.next_epoch == validator.withdrawable_epoch {
        validator.exit_cache_index.safe_add_assign(1)?;
        validator.exit_epoch = state_ctxt.next_epoch.safe_add(spec.min_validator_exit_epochs)?;
    }

    let is_eligible = info.is_eligible || activation_queue.is_validator_eligible_for_activation(info.index);
    if is_eligible && validator.is_active_at(state_ctxt.next_epoch) {
        next_epoch_activation_queue.add_validator(info.index);
    }

    Ok(())
}

/// Updates the effective balance for a single validator.
fn process_single_effective_balance_update(
    balance: u64,
    validator: &mut Cow<Validator>,
    info: &ValidatorInfo,
    next_epoch_total_active_balance: &mut u64,
    next_epoch_cache: &mut PreEpochCache,
    progressive_balances: &ProgressiveBalancesCache,
    effective_balances_ctxt: &EffectiveBalancesContext,
    state_ctxt: &StateContext,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let new_effective_balance = validator.effective_balance
        .safe_add(state_ctxt.churn_limit)?
        .safe_add(progressive_balances.total_balance_next_epoch(info.index))?
        .safe_sub(info.effective_balance)?
        .safe_div(spec.effective_balance_increment)?
        .safe_mul(spec.effective_balance_increment)?;
    validator.effective_balance = min(new_effective_balance, spec.max_effective_balance);

    if validator.is_active_at(state_ctxt.next_epoch) {
        *next_epoch_total_active_balance = next_epoch_total_active_balance
            .safe_add(validator.effective_balance)?;
    }

    next_epoch_cache.update_for_validator(info.index, validator)?;

    Ok(())
}

/// Handles a slashing for a single validator.
fn process_single_slashing(
    balance: &mut Cow<u64>,
    validator: &Validator,
    slashings_ctxt: &SlashingsContext,
    state_ctxt: &StateContext,
    spec: &ChainSpec,
) -> Result<(), Error> {
    let penalty = min(
        validator.effective_balance.safe_div(2)?,
        validator.slashed_balance,
    );

    *balance.safe_sub_assign(penalty)?;
    slashings_ctxt.adjusted_total_slashing_balance.safe_add_assign(penalty)?;

    Ok(())
}

/// Initializes the `SlashingsContext`.
impl SlashingsContext {
    fn new<E: EthSpec>(
        state: &BeaconState<E>,
        state_ctxt: &StateContext,
        spec: &ChainSpec,
    ) -> Result<Self, Error> {
        let adjusted_total_slashing_balance = state
            .slashings
            .iter()
            .try_fold(0, |sum, &x| sum.safe_add(x))?
            .safe_mul(3)?
            .safe_div(2)?;

        let target_withdrawable_epoch = state_ctxt.next_epoch.safe_add(spec.min_validator_withdrawable_epochs)?;

        Ok(Self {
            adjusted_total_slashing_balance,
            target_withdrawable_epoch,
        })
    }
}

/// Initializes the `RewardsAndPenaltiesContext`.
impl RewardsAndPenaltiesContext {
    fn new(
        progressive_balances: &ProgressiveBalancesCache,
        state_ctxt: &StateContext,
        spec: &ChainSpec,
    ) -> Result<Self, Error> {
        let mut unslashed_participating_increments_array = [0; NUM_FLAG_INDICES];
        for flag_index in 0..NUM_FLAG_INDICES {
            let progressive_balance = progressive_balances
                .get_progressive_balance(state_ctxt.current_epoch, flag_index)?;
            unslashed_participating_increments_array[flag_index] = progressive_balance
                .safe_div(spec.effective_balance_increment)?;
        }

        let active_increments = state_ctxt
            .total_active_balance
            .safe_div(spec.effective_balance_increment)?;

        Ok(Self {
            unslashed_participating_increments_array,
            active_increments,
        })
    }
}

/// Initializes the `EffectiveBalancesContext`.
impl EffectiveBalancesContext {
    fn new(spec: &ChainSpec) -> Result<Self, Error> {
        let downward_threshold = spec.max_effective_balance.safe_sub(spec.effective_balance_increment)?;
        let upward_threshold = spec.max_effective_balance.safe_sub(spec.effective_balance_increment)?;

        Ok(Self {
            downward_threshold,
            upward_threshold,
        })
    }
}
