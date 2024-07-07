use safe_arith::ArithError;
use types::{Checkpoint, Epoch, ExecutionBlockHash, Hash256, Slot};

/// Errors that can occur in the validation and processing of blockchain nodes.
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    /// The node with the given `Hash256` is unknown in the finalized state.
    FinalizedNodeUnknown(Hash256),
    /// The node with the given `Hash256` is unknown in the justified state.
    JustifiedNodeUnknown(Hash256),
    /// The node with the given `Hash256` is unknown.
    NodeUnknown(Hash256),
    /// The attempted change of the finalized root was invalid.
    InvalidFinalizedRootChange,
    /// The index provided for a node is invalid.
    InvalidNodeIndex(usize),
    /// The index provided for a parent node is invalid.
    InvalidParentIndex(usize),
    /// The index provided for the best child node is invalid.
    InvalidBestChildIndex(usize),
    /// The index provided for the justified node is invalid.
    InvalidJustifiedIndex(usize),
    /// The index provided for the best descendant node is invalid.
    InvalidBestDescendant(usize),
    /// The delta value provided for the parent node is invalid.
    InvalidParentDelta(usize),
    /// The delta value provided for the node is invalid.
    InvalidNodeDelta(usize),
    /// The justified checkpoint is missing.
    MissingJustifiedCheckpoint,
    /// The finalized checkpoint is missing.
    MissingFinalizedCheckpoint,
    /// An overflow occurred during delta calculation.
    DeltaOverflow(usize),
    /// An overflow occurred during proposer boost calculation.
    ProposerBoostOverflow(usize),
    /// An overflow occurred during reorganization threshold calculation.
    ReOrgThresholdOverflow,
    /// An overflow occurred in an index operation with a specific error message.
    IndexOverflow(&'static str),
    /// An overflow occurred during invalid execution delta calculation.
    InvalidExecutionDeltaOverflow(usize),
    /// The length of deltas or indices is invalid.
    InvalidDeltaLen {
        /// Number of deltas provided.
        deltas: usize,
        /// Number of indices provided.
        indices: usize,
    },
    /// The finalized epoch was reverted from `current_finalized_epoch` to `new_finalized_epoch`.
    RevertedFinalizedEpoch {
        /// Current finalized epoch.
        current_finalized_epoch: Epoch,
        /// New finalized epoch.
        new_finalized_epoch: Epoch,
    },
    /// The best node is invalid with detailed information.
    InvalidBestNode(Box<InvalidBestNodeInfo>),
    /// An ancestor of a valid payload is invalid.
    InvalidAncestorOfValidPayload {
        /// Ancestor block root.
        ancestor_block_root: Hash256,
        /// Execution block hash of the ancestor payload.
        ancestor_payload_block_hash: ExecutionBlockHash,
    },
    /// The execution status changed from valid to invalid for a block.
    ValidExecutionStatusBecameInvalid {
        /// Block root hash.
        block_root: Hash256,
        /// Execution block hash of the payload.
        payload_block_hash: ExecutionBlockHash,
    },
    /// The execution status of the justified checkpoint is invalid.
    InvalidJustifiedCheckpointExecutionStatus {
        /// Justified root hash.
        justified_root: Hash256,
    },
    /// The latest valid ancestor hash for a block root is unknown.
    UnknownLatestValidAncestorHash {
        /// Block root hash.
        block_root: Hash256,
        /// Optional latest valid ancestor execution block hash.
        latest_valid_ancestor_hash: Option<ExecutionBlockHash>,
    },
    /// The descendant block root is irrelevant.
    IrrelevantDescendant {
        /// Block root hash.
        block_root: Hash256,
    },
    /// The execution status of the parent block is invalid.
    ParentExecutionStatusIsInvalid {
        /// Block root hash.
        block_root: Hash256,
        /// Parent block root hash.
        parent_root: Hash256,
    },
    /// The epoch offset provided is invalid.
    InvalidEpochOffset(u64),
    /// Arithmetic error occurred.
    Arith(ArithError),
}

impl From<ArithError> for Error {
    /// Converts an `ArithError` into an `Error`.
    fn from(e: ArithError) -> Self {
        Error::Arith(e)
    }
}

/// Information about an invalid best node.
#[derive(Clone, PartialEq, Debug)]
pub struct InvalidBestNodeInfo {
    /// Current slot.
    pub current_slot: Slot,
    /// Start root hash.
    pub start_root: Hash256,
    /// Justified checkpoint.
    pub justified_checkpoint: Checkpoint,
    /// Finalized checkpoint.
    pub finalized_checkpoint: Checkpoint,
    /// Head root hash.
    pub head_root: Hash256,
    /// Head justified checkpoint.
    pub head_justified_checkpoint: Checkpoint,
    /// Head finalized checkpoint.
    pub head_finalized_checkpoint: Checkpoint,
}
