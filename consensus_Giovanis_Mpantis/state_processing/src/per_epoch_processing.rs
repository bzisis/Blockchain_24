#![deny(clippy::wildcard_imports)]

use crate::metrics;
pub use epoch_processing_summary::{EpochProcessingSummary, ParticipationEpochSummary};
use errors::EpochProcessingError as Error;
pub use justification_and_finalization_state::JustificationAndFinalizationState;
use safe_arith::SafeArith;
use types::{BeaconState, ChainSpec, EthSpec};

pub use registry_updates::{process_registry_updates, process_registry_updates_slow};
pub use slashings::{process_slashings, process_slashings_slow};
pub use weigh_justification_and_finalization::weigh_justification_and_finalization;

pub mod altair;
pub mod base;
pub mod capella;
pub mod effective_balance_updates;
pub mod epoch_processing_summary;
pub mod errors;
pub mod historical_roots_update;
pub mod justification_and_finalization_state;
pub mod registry_updates;
pub mod resets;
pub mod single_pass;
pub mod slashings;
pub mod tests;
pub mod weigh_justification_and_finalization;

/// Performs per-epoch processing on some BeaconState.
///
/// This function mutates the given `BeaconState`, returning an `EpochProcessingSummary` on success
/// or an `Error` if processing fails midway.
///
/// # Errors
///
/// Returns an `Error` if the `BeaconState` instantiation does not match the fork at `state.slot()`.
pub fn process_epoch<E: EthSpec>(
    state: &mut BeaconState<E>,
    spec: &ChainSpec,
) -> Result<EpochProcessingSummary<E>, Error> {
    let _timer = metrics::start_timer(&metrics::PROCESS_EPOCH_TIME);

    // Verify that the `BeaconState` instantiation matches the fork at `state.slot()`.
    state
        .fork_name(spec)
        .map_err(Error::InconsistentStateFork)?;

    match state {
        BeaconState::Base(_) => base::process_epoch(state, spec),
        BeaconState::Altair(_)
        | BeaconState::Bellatrix(_)
        | BeaconState::Capella(_)
        | BeaconState::Deneb(_)
        | BeaconState::Electra(_) => altair::process_epoch(state, spec),
    }
}

/// Used to track the changes to a validator's balance.
#[derive(Default, Clone)]
pub struct Delta {
    /// The total rewards accumulated.
    pub rewards: u64,
    /// The total penalties accumulated.
    pub penalties: u64,
}

impl Delta {
    /// Reward the validator with the given `reward`.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if adding the `reward` causes overflow.
    pub fn reward(&mut self, reward: u64) -> Result<(), Error> {
        self.rewards = self.rewards.safe_add(reward)?;
        Ok(())
    }

    /// Penalize the validator with the given `penalty`.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if adding the `penalty` causes overflow.
    pub fn penalize(&mut self, penalty: u64) -> Result<(), Error> {
        self.penalties = self.penalties.safe_add(penalty)?;
        Ok(())
    }

    /// Combine two `Delta` structures into one.
    ///
    /// This method adds the rewards and penalties from `other` to `self`.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if adding rewards or penalties causes overflow.
    fn combine(&mut self, other: Delta) -> Result<(), Error> {
        self.reward(other.rewards)?;
        self.penalize(other.penalties)
    }
}
