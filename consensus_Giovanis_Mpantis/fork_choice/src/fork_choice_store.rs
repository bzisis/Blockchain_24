use proto_array::JustifiedBalances;
use std::collections::BTreeSet;
use std::fmt::Debug;
use types::{AbstractExecPayload, BeaconBlockRef, BeaconState, Checkpoint, EthSpec, Hash256, Slot};

/// Approximates the `Store` in "Ethereum 2.0 Phase 0 -- Beacon Chain Fork Choice":
///
/// https://github.com/ethereum/eth2.0-specs/blob/v0.12.1/specs/phase0/fork-choice.md#store
///
/// ## Detail
///
/// This trait defines methods for managing fork choice data, providing an abstraction
/// to allow different implementations (e.g., in-memory vs. database-backed).
///
/// - The block DAG is managed in `ProtoArrayForkChoice`.
/// - Time is represented using `Slot` instead of UNIX epoch `u64`.
///
/// ## Motivation
///
/// This trait separates data management concerns from fork choice logic,
/// promoting auditability and flexibility in implementation choices.
pub trait ForkChoiceStore<E: EthSpec>: Sized {
    type Error: Debug;

    /// Returns the current slot value.
    ///
    /// This method retrieves the last slot value set by `set_current_slot`.
    fn get_current_slot(&self) -> Slot;

    /// Sets the current slot value.
    ///
    /// This method updates the current slot value to track the latest block processing.
    /// It should only be called from within `ForkChoice::on_tick`.
    fn set_current_slot(&mut self, slot: Slot);

    /// Called when a block has been verified but not yet added to fork choice.
    ///
    /// This method allows implementing caching or other preparatory tasks.
    fn on_verified_block<Payload: AbstractExecPayload<E>>(
        &mut self,
        block: BeaconBlockRef<E, Payload>,
        block_root: Hash256,
        state: &BeaconState<E>,
    ) -> Result<(), Self::Error>;

    /// Returns the justified checkpoint.
    ///
    /// This method retrieves the checkpoint indicating the justified state.
    fn justified_checkpoint(&self) -> &Checkpoint;

    /// Returns the justified balances associated with the justified checkpoint.
    ///
    /// This method retrieves the balances from the state identified by the justified checkpoint.
    fn justified_balances(&self) -> &JustifiedBalances;

    /// Returns the finalized checkpoint.
    ///
    /// This method retrieves the checkpoint indicating the finalized state.
    fn finalized_checkpoint(&self) -> &Checkpoint;

    /// Returns the unrealized justified checkpoint.
    ///
    /// This method retrieves the checkpoint representing a proposed but not yet accepted justification.
    fn unrealized_justified_checkpoint(&self) -> &Checkpoint;

    /// Returns the unrealized finalized checkpoint.
    ///
    /// This method retrieves the checkpoint representing a proposed but not yet accepted finalization.
    fn unrealized_finalized_checkpoint(&self) -> &Checkpoint;

    /// Returns the root of the proposer boost.
    ///
    /// This method retrieves the root hash associated with proposer boosting.
    fn proposer_boost_root(&self) -> Hash256;

    /// Sets the finalized checkpoint.
    ///
    /// This method updates the finalized checkpoint to a new value.
    fn set_finalized_checkpoint(&mut self, checkpoint: Checkpoint);

    /// Sets the justified checkpoint.
    ///
    /// This method updates the justified checkpoint to a new value.
    fn set_justified_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), Self::Error>;

    /// Sets the unrealized justified checkpoint.
    ///
    /// This method updates the proposed but not yet accepted justified checkpoint.
    fn set_unrealized_justified_checkpoint(&mut self, checkpoint: Checkpoint);

    /// Sets the unrealized finalized checkpoint.
    ///
    /// This method updates the proposed but not yet accepted finalized checkpoint.
    fn set_unrealized_finalized_checkpoint(&mut self, checkpoint: Checkpoint);

    /// Sets the root of the proposer boost.
    ///
    /// This method updates the root hash associated with proposer boosting.
    fn set_proposer_boost_root(&mut self, proposer_boost_root: Hash256);

    /// Retrieves the set of equivocating validator indices.
    ///
    /// This method retrieves the indices of validators who have equivocated.
    fn equivocating_indices(&self) -> &BTreeSet<u64>;

    /// Extends the set of equivocating validator indices.
    ///
    /// This method adds multiple indices to the set of validators who have equivocated.
    fn extend_equivocating_indices(&mut self, indices: impl IntoIterator<Item = u64>);
}
