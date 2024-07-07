use types::{BeaconState, BeaconStateError, BitVector, Checkpoint, Epoch, EthSpec, Hash256};

/// This is a subset of the `BeaconState` which is used to compute justification and finality
/// without modifying the `BeaconState`.
///
/// A `JustificationAndFinalizationState` can be created from a `BeaconState` to compute
/// justification/finality changes and then applied to a `BeaconState` to enshrine those changes.
#[must_use = "this value must be applied to a state or explicitly dropped"]
pub struct JustificationAndFinalizationState<E: EthSpec> {
    /*
     * Immutable fields.
     */
    /// Previous epoch of the beacon chain state.
    previous_epoch: Epoch,
    /// Root of the beacon chain state at the previous epoch.
    previous_epoch_target_root: Result<Hash256, BeaconStateError>,
    /// Current epoch of the beacon chain state.
    current_epoch: Epoch,
    /// Root of the beacon chain state at the current epoch.
    current_epoch_target_root: Result<Hash256, BeaconStateError>,
    /*
     * Mutable fields.
     */
    /// Checkpoint marking the last justified epoch in the beacon chain state.
    previous_justified_checkpoint: Checkpoint,
    /// Checkpoint marking the current justified epoch in the beacon chain state.
    current_justified_checkpoint: Checkpoint,
    /// Checkpoint marking the last finalized epoch in the beacon chain state.
    finalized_checkpoint: Checkpoint,
    /// Vector of justification bits, representing the state's justification status.
    justification_bits: BitVector<E::JustificationBitsLength>,
}

impl<E: EthSpec> JustificationAndFinalizationState<E> {
    /// Creates a new `JustificationAndFinalizationState` from a given `BeaconState`.
    pub fn new(state: &BeaconState<E>) -> Self {
        let previous_epoch = state.previous_epoch();
        let current_epoch = state.current_epoch();
        Self {
            previous_epoch,
            previous_epoch_target_root: state.get_block_root_at_epoch(previous_epoch).copied(),
            current_epoch,
            current_epoch_target_root: state.get_block_root_at_epoch(current_epoch).copied(),
            previous_justified_checkpoint: state.previous_justified_checkpoint(),
            current_justified_checkpoint: state.current_justified_checkpoint(),
            finalized_checkpoint: state.finalized_checkpoint(),
            justification_bits: state.justification_bits().clone(),
        }
    }

    /// Applies changes from this state to a given `BeaconState`.
    pub fn apply_changes_to_state(self, state: &mut BeaconState<E>) {
        let Self {
            previous_justified_checkpoint,
            current_justified_checkpoint,
            finalized_checkpoint,
            justification_bits,
            ..
        } = self;

        *state.previous_justified_checkpoint_mut() = previous_justified_checkpoint;
        *state.current_justified_checkpoint_mut() = current_justified_checkpoint;
        *state.finalized_checkpoint_mut() = finalized_checkpoint;
        *state.justification_bits_mut() = justification_bits;
    }

    /// Returns the previous epoch of the state.
    pub fn previous_epoch(&self) -> Epoch {
        self.previous_epoch
    }

    /// Returns the current epoch of the state.
    pub fn current_epoch(&self) -> Epoch {
        self.current_epoch
    }

    /// Retrieves the block root at a specified epoch.
    ///
    /// Returns the root of the block at the specified epoch or an error if the epoch is out of bounds.
    pub fn get_block_root_at_epoch(&self, epoch: Epoch) -> Result<Hash256, BeaconStateError> {
        if epoch == self.previous_epoch {
            self.previous_epoch_target_root.clone()
        } else if epoch == self.current_epoch {
            self.current_epoch_target_root.clone()
        } else {
            Err(BeaconStateError::SlotOutOfBounds)
        }
    }

    /// Returns the previous justified checkpoint.
    pub fn previous_justified_checkpoint(&self) -> Checkpoint {
        self.previous_justified_checkpoint
    }

    /// Returns a mutable reference to the previous justified checkpoint.
    pub fn previous_justified_checkpoint_mut(&mut self) -> &mut Checkpoint {
        &mut self.previous_justified_checkpoint
    }

    /// Returns a mutable reference to the current justified checkpoint.
    pub fn current_justified_checkpoint_mut(&mut self) -> &mut Checkpoint {
        &mut self.current_justified_checkpoint
    }

    /// Returns the current justified checkpoint.
    pub fn current_justified_checkpoint(&self) -> Checkpoint {
        self.current_justified_checkpoint
    }

    /// Returns the finalized checkpoint.
    pub fn finalized_checkpoint(&self) -> Checkpoint {
        self.finalized_checkpoint
    }

    /// Returns a mutable reference to the finalized checkpoint.
    pub fn finalized_checkpoint_mut(&mut self) -> &mut Checkpoint {
        &mut self.finalized_checkpoint
    }

    /// Returns a reference to the justification bits.
    pub fn justification_bits(&self) -> &BitVector<E::JustificationBitsLength> {
        &self.justification_bits
    }

    /// Returns a mutable reference to the justification bits.
    pub fn justification_bits_mut(&mut self) -> &mut BitVector<E::JustificationBitsLength> {
        &mut self.justification_bits
    }
}
