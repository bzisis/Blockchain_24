use super::signature_sets::Error as SignatureSetError;
use crate::ContextError;
use merkle_proof::MerkleTreeError;
use safe_arith::ArithError;
use ssz::DecodeError;
use types::*;

/// The error returned from the `per_block_processing` function. Indicates that a block is either
/// invalid, or we were unable to determine its validity (we encountered an unexpected error).
///
/// Any of the `...Error` variants indicate that at some point during block (and block operation)
/// verification, there was an error. There is no indication as to _where_ that error happened
/// (e.g., when processing attestations instead of when processing deposits).
#[derive(Debug, PartialEq, Clone)]
pub enum BlockProcessingError {
    /// Logic error indicating that the wrong state type was provided.
    IncorrectStateType,
    RandaoSignatureInvalid,
    BulkSignatureVerificationFailed,
    StateRootMismatch,
    DepositCountInvalid {
        /// Expected number of deposits.
        expected: usize,
        /// Found number of deposits.
        found: usize,
    },
    HeaderInvalid {
        /// Reason why the header is invalid.
        reason: HeaderInvalid,
    },
    ProposerSlashingInvalid {
        /// Index of the proposer slashing.
        index: usize,
        /// Reason why the proposer slashing is invalid.
        reason: ProposerSlashingInvalid,
    },
    AttesterSlashingInvalid {
        /// Index of the attester slashing.
        index: usize,
        /// Reason why the attester slashing is invalid.
        reason: AttesterSlashingInvalid,
    },
    IndexedAttestationInvalid {
        /// Index of the indexed attestation.
        index: usize,
        /// Reason why the indexed attestation is invalid.
        reason: IndexedAttestationInvalid,
    },
    AttestationInvalid {
        /// Index of the attestation.
        index: usize,
        /// Reason why the attestation is invalid.
        reason: AttestationInvalid,
    },
    DepositInvalid {
        /// Index of the deposit.
        index: usize,
        /// Reason why the deposit is invalid.
        reason: DepositInvalid,
    },
    ExitInvalid {
        /// Index of the exit.
        index: usize,
        /// Reason why the exit is invalid.
        reason: ExitInvalid,
    },
    BlsExecutionChangeInvalid {
        /// Index of the BLS execution change.
        index: usize,
        /// Reason why the BLS execution change is invalid.
        reason: BlsExecutionChangeInvalid,
    },
    SyncAggregateInvalid {
        /// Reason why the sync aggregate is invalid.
        reason: SyncAggregateInvalid,
    },
    BeaconStateError(BeaconStateError),
    SignatureSetError(SignatureSetError),
    SszTypesError(ssz_types::Error),
    SszDecodeError(DecodeError),
    MerkleTreeError(MerkleTreeError),
    ArithError(ArithError),
    InconsistentBlockFork(InconsistentFork),
    InconsistentStateFork(InconsistentFork),
    ExecutionHashChainIncontiguous {
        /// Expected execution block hash.
        expected: ExecutionBlockHash,
        /// Found execution block hash.
        found: ExecutionBlockHash,
    },
    ExecutionRandaoMismatch {
        /// Expected Randao value.
        expected: Hash256,
        /// Found Randao value.
        found: Hash256,
    },
    ExecutionInvalidTimestamp {
        /// Expected timestamp value.
        expected: u64,
        /// Found timestamp value.
        found: u64,
    },
    ExecutionInvalidBlobsLen {
        /// Maximum allowed length for blobs.
        max: usize,
        /// Actual length of blobs.
        actual: usize,
    },
    ExecutionInvalid,
    ConsensusContext(ContextError),
    MilhouseError(milhouse::Error),
    EpochCacheError(EpochCacheError),
    WithdrawalsRootMismatch {
        /// Expected withdrawals root hash.
        expected: Hash256,
        /// Found withdrawals root hash.
        found: Hash256,
    },
    WithdrawalCredentialsInvalid,
}

impl From<BeaconStateError> for BlockProcessingError {
    fn from(e: BeaconStateError) -> Self {
        BlockProcessingError::BeaconStateError(e)
    }
}

impl From<SignatureSetError> for BlockProcessingError {
    fn from(e: SignatureSetError) -> Self {
        BlockProcessingError::SignatureSetError(e)
    }
}

impl From<ssz_types::Error> for BlockProcessingError {
    fn from(error: ssz_types::Error) -> Self {
        BlockProcessingError::SszTypesError(error)
    }
}

impl From<DecodeError> for BlockProcessingError {
    fn from(error: DecodeError) -> Self {
        BlockProcessingError::SszDecodeError(error)
    }
}

impl From<ArithError> for BlockProcessingError {
    fn from(e: ArithError) -> Self {
        BlockProcessingError::ArithError(e)
    }
}

impl From<SyncAggregateInvalid> for BlockProcessingError {
    fn from(reason: SyncAggregateInvalid) -> Self {
        BlockProcessingError::SyncAggregateInvalid { reason }
    }
}

impl From<ContextError> for BlockProcessingError {
    fn from(e: ContextError) -> Self {
        BlockProcessingError::ConsensusContext(e)
    }
}

impl From<EpochCacheError> for BlockProcessingError {
    fn from(e: EpochCacheError) -> Self {
        BlockProcessingError::EpochCacheError(e)
    }
}

impl From<milhouse::Error> for BlockProcessingError {
    fn from(e: milhouse::Error) -> Self {
        Self::MilhouseError(e)
    }
}

impl From<BlockOperationError<HeaderInvalid>> for BlockProcessingError {
    fn from(e: BlockOperationError<HeaderInvalid>) -> BlockProcessingError {
        match e {
            BlockOperationError::Invalid(reason) => BlockProcessingError::HeaderInvalid { reason },
            BlockOperationError::BeaconStateError(e) => BlockProcessingError::BeaconStateError(e),
            BlockOperationError::SignatureSetError(e) => BlockProcessingError::SignatureSetError(e),
            BlockOperationError::SszTypesError(e) => BlockProcessingError::SszTypesError(e),
            BlockOperationError::ConsensusContext(e) => BlockProcessingError::ConsensusContext(e),
            BlockOperationError::ArithError(e) => BlockProcessingError::ArithError(e),
        }
    }
}

/// A conversion that consumes `self` and adds an `index` variable to resulting struct.
///
/// Used here to allow converting an error into an upstream error that points to the object that
/// caused the error. For example, pointing to the index of an attestation that caused the
/// `AttestationInvalid` error.
pub trait IntoWithIndex<T>: Sized {
    fn into_with_index(self, index: usize) -> T;
}

macro_rules! impl_into_block_processing_error_with_index {
    ($($type: ident),*) => {
        $(
            impl IntoWithIndex<BlockProcessingError> for BlockOperationError<$type> {
                fn into_with_index(self, index: usize) -> BlockProcessingError {
                    match self {
                        BlockOperationError::Invalid(reason) => BlockProcessingError::$type {
                            index,
                            reason
                        },
                        BlockOperationError::BeaconStateError(e) => BlockProcessingError::BeaconStateError(e),
                        BlockOperationError::SignatureSetError(e) => BlockProcessingError::SignatureSetError(e),
                        BlockOperationError::SszTypesError(e) => BlockProcessingError::SszTypesError(e),
                        BlockOperationError::ConsensusContext(e) => BlockProcessingError::ConsensusContext(e),
                        BlockOperationError::ArithError(e) => BlockProcessingError::ArithError(e),
                    }
                }
            }
        )*
    };
}

impl_into_block_processing_error_with_index!(
    ProposerSlashingInvalid,
    AttesterSlashingInvalid,
    IndexedAttestationInvalid,
    AttestationInvalid,
    DepositInvalid,
    ExitInvalid,
    BlsExecutionChangeInvalid
);

pub type HeaderValidationError = BlockOperationError<HeaderInvalid>;
pub type AttesterSlashingValidationError = BlockOperationError<AttesterSlashingInvalid>;
pub type ProposerSlashingValidationError = BlockOperationError<ProposerSlashingInvalid>;
pub type AttestationValidationError = BlockOperationError<AttestationInvalid>;
pub type SyncCommitteeMessageValidationError = BlockOperationError<SyncAggregateInvalid>;
pub type DepositValidationError = BlockOperationError<DepositInvalid>;
pub type ExitValidationError = BlockOperationError<ExitInvalid>;
pub type BlsExecutionChangeValidationError = BlockOperationError<BlsExecutionChangeInvalid>;

#[derive(Debug, PartialEq, Clone)]
pub enum BlockOperationError<T> {
    /// Indicates that an operation on a block was invalid.
    Invalid(T),
    BeaconStateError(BeaconStateError),
    SignatureSetError(SignatureSetError),
    SszTypesError(ssz_types::Error),
    ConsensusContext(ContextError),
    ArithError(ArithError),
}

impl<T> BlockOperationError<T> {
    /// Creates a new `Invalid` variant with the given reason.
    pub fn invalid(reason: T) -> BlockOperationError<T> {
        BlockOperationError::Invalid(reason)
    }
}

impl<T> From<BeaconStateError> for BlockOperationError<T> {
    fn from(e: BeaconStateError) -> Self {
        BlockOperationError::BeaconStateError(e)
    }
}

impl<T> From<SignatureSetError> for BlockOperationError<T> {
    fn from(e: SignatureSetError) -> Self {
        BlockOperationError::SignatureSetError(e)
    }
}

impl<T> From<ssz_types::Error> for BlockOperationError<T> {
    fn from(error: ssz_types::Error) -> Self {
        BlockOperationError::SszTypesError(error)
    }
}

impl<T> From<ArithError> for BlockOperationError<T> {
    fn from(e: ArithError) -> Self {
        BlockOperationError::ArithError(e)
    }
}

impl<T> From<ContextError> for BlockOperationError<T> {
    fn from(e: ContextError) -> Self {
        BlockOperationError::ConsensusContext(e)
    }
}

/// Reasons why a header might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum HeaderInvalid {
    ProposalSignatureInvalid,
    StateSlotMismatch,
    OlderThanLatestBlockHeader {
        /// Latest block header slot.
        latest_block_header_slot: Slot,
        /// Block slot.
        block_slot: Slot,
    },
    ProposerIndexMismatch {
        /// Block proposer index.
        block_proposer_index: u64,
        /// State proposer index.
        state_proposer_index: u64,
    },
    ParentBlockRootMismatch {
        /// State block root.
        state: Hash256,
        /// Block root.
        block: Hash256,
    },
    ProposerSlashed(u64),
}

/// Reasons why a proposer slashing might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum ProposerSlashingInvalid {
    /// The proposer index is not a known validator.
    ProposerUnknown(u64),
    /// The two proposals have different slots.
    ///
    /// (proposal_1_slot, proposal_2_slot)
    ProposalSlotMismatch(Slot, Slot),
    /// The two proposals have different proposer indices.
    ///
    /// (proposer_index_1, proposer_index_2)
    ProposerIndexMismatch(u64, u64),
    /// The proposals are identical and therefore not slashable.
    ProposalsIdentical,
    /// The specified proposer cannot be slashed because they are already slashed, or not active.
    ProposerNotSlashable(u64),
    /// The first proposal signature was invalid.
    BadProposal1Signature,
    /// The second proposal signature was invalid.
    BadProposal2Signature,
}

/// Reasons why an attester slashing might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum AttesterSlashingInvalid {
    /// The attestations were not in conflict.
    NotSlashable,
    /// The first `IndexedAttestation` was invalid.
    IndexedAttestation1Invalid(BlockOperationError<IndexedAttestationInvalid>),
    /// The second `IndexedAttestation` was invalid.
    IndexedAttestation2Invalid(BlockOperationError<IndexedAttestationInvalid>),
    /// The validator index is unknown. One cannot slash one who does not exist.
    UnknownValidator(u64),
    /// There were no indices able to be slashed.
    NoSlashableIndices,
}

/// Reasons why an attestation might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum AttestationInvalid {
    /// Committee index exceeds number of committees in that slot.
    BadCommitteeIndex,
    /// Attestation included before the inclusion delay.
    IncludedTooEarly {
        /// State slot.
        state: Slot,
        /// Delay.
        delay: u64,
        /// Attestation slot.
        attestation: Slot,
    },
    /// Attestation slot is too far in the past to be included in a block.
    IncludedTooLate {
        /// State slot.
        state: Slot,
        /// Attestation slot.
        attestation: Slot,
    },
    /// Attestation target epoch does not match attestation slot.
    TargetEpochSlotMismatch {
        /// Target epoch.
        target_epoch: Epoch,
        /// Slot epoch.
        slot_epoch: Epoch,
    },
    /// Attestation target epoch does not match the current or previous epoch.
    BadTargetEpoch,
    /// Attestation justified checkpoint doesn't match the state's current or previous justified
    /// checkpoint.
    ///
    /// `is_current` is `true` if the attestation was compared to the
    /// `state.current_justified_checkpoint`, `false` if compared to `state.previous_justified_checkpoint`.
    ///
    /// Checkpoints have been boxed to keep the error size down and prevent clippy failures.
    WrongJustifiedCheckpoint {
        /// State checkpoint.
        state: Box<Checkpoint>,
        /// Attestation checkpoint.
        attestation: Box<Checkpoint>,
        /// Indicates if the comparison is with the current justified checkpoint.
        is_current: bool,
    },
    /// The aggregation bitfield length is not the smallest possible size to represent the committee.
    BadAggregationBitfieldLength {
        /// Committee length.
        committee_len: usize,
        /// Bitfield length.
        bitfield_len: usize,
    },
    /// The attestation was not disjoint compared to already seen attestations.
    NotDisjoint,
    /// The validator index was unknown.
    UnknownValidator(u64),
    /// The attestation signature verification failed.
    BadSignature,
    /// The indexed attestation created from this attestation was found to be invalid.
    BadIndexedAttestation(IndexedAttestationInvalid),
}

impl From<BlockOperationError<IndexedAttestationInvalid>>
    for BlockOperationError<AttestationInvalid>
{
    fn from(e: BlockOperationError<IndexedAttestationInvalid>) -> Self {
        match e {
            BlockOperationError::Invalid(e) => {
                BlockOperationError::invalid(AttestationInvalid::BadIndexedAttestation(e))
            }
            BlockOperationError::BeaconStateError(e) => BlockOperationError::BeaconStateError(e),
            BlockOperationError::SignatureSetError(e) => BlockOperationError::SignatureSetError(e),
            BlockOperationError::SszTypesError(e) => BlockOperationError::SszTypesError(e),
            BlockOperationError::ConsensusContext(e) => BlockOperationError::ConsensusContext(e),
            BlockOperationError::ArithError(e) => BlockOperationError::ArithError(e),
        }
    }
}

/// Reasons why an indexed attestation might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum IndexedAttestationInvalid {
    /// The number of indices is 0.
    IndicesEmpty,
    /// The validator indices were not in increasing order.
    ///
    /// The error occurred between the given `index` and `index + 1`
    BadValidatorIndicesOrdering(usize),
    /// The validator index is unknown. One cannot slash one who does not exist.
    UnknownValidator(u64),
    /// The indexed attestation aggregate signature was not valid.
    BadSignature,
    /// There was an error whilst attempting to get a set of signatures. The signatures may have
    /// been invalid or an internal error occurred.
    SignatureSetError(SignatureSetError),
}

/// Reasons why a deposit might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum DepositInvalid {
    /// The signature (proof-of-possession) does not match the given pubkey.
    BadSignature,
    /// The signature or pubkey does not represent a valid BLS point.
    BadBlsBytes,
    /// The specified `branch` and `index` did not form a valid proof that the deposit is included
    /// in the eth1 deposit root.
    BadMerkleProof,
}

/// Reasons why an exit might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum ExitInvalid {
    /// The specified validator is not active.
    NotActive(u64),
    /// The specified validator is not in the state's validator registry.
    ValidatorUnknown(u64),
    /// The specified validator has a non-maximum exit epoch.
    AlreadyExited(u64),
    /// The specified validator has already initiated exit.
    AlreadyInitiatedExit(u64),
    /// The exit is for a future epoch.
    FutureEpoch {
        /// Current epoch.
        current_epoch: Epoch,
        /// Earliest exit epoch.
        earliest_exit_epoch: Epoch,
    },
    /// The exit signature was not signed by the validator.
    BadSignature,
    /// There was an error whilst attempting to get a set of signatures. The signatures may have
    /// been invalid or an internal error occurred.
    SignatureSetError(SignatureSetError),
}

/// Reasons why a BLS execution change might be invalid.
#[derive(Debug, PartialEq, Clone)]
pub enum BlsExecutionChangeInvalid {
    /// The specified validator is not in the state's validator registry.
    ValidatorUnknown(u64),
    /// Validator
