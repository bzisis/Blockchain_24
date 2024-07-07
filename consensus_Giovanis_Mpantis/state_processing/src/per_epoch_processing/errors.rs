use types::{milhouse, BeaconStateError, EpochCacheError, InconsistentFork};

/// Errors that can occur during epoch processing.
#[derive(Debug, PartialEq)]
pub enum EpochProcessingError {
    /// Unable to determine the producer for the epoch.
    UnableToDetermineProducer,
    /// No block roots were found.
    NoBlockRoots,
    /// The base reward quotient is zero.
    BaseRewardQuotientIsZero,
    /// No RANDAO seed was found.
    NoRandaoSeed,
    /// The previous total balance is zero.
    PreviousTotalBalanceIsZero,
    /// The inclusion distance is zero.
    InclusionDistanceZero,
    /// Validator statuses are inconsistent.
    ValidatorStatusesInconsistent,
    /// Deltas are inconsistent.
    DeltasInconsistent,
    /// The delta value is out of bounds.
    DeltaOutOfBounds(usize),
    /// Unable to get the inclusion distance for a validator that should have an inclusion
    /// distance. This indicates an internal inconsistency.
    ///
    /// (validator_index)
    InclusionSlotsInconsistent(usize),
    /// An error related to the beacon state.
    BeaconStateError(BeaconStateError),
    /// An error related to inclusion.
    InclusionError(InclusionError),
    /// An error related to SSZ types.
    SszTypesError(ssz_types::Error),
    /// An arithmetic error.
    ArithError(safe_arith::ArithError),
    /// The state fork is inconsistent.
    InconsistentStateFork(InconsistentFork),
    /// The justification bit is invalid.
    InvalidJustificationBit(ssz_types::Error),
    /// The flag index is invalid.
    InvalidFlagIndex(usize),
    /// An error from Milhouse.
    MilhouseError(milhouse::Error),
    /// An error related to the epoch cache.
    EpochCache(EpochCacheError),
}

impl From<InclusionError> for EpochProcessingError {
    /// Converts an `InclusionError` into an `EpochProcessingError`.
    fn from(e: InclusionError) -> EpochProcessingError {
        EpochProcessingError::InclusionError(e)
    }
}

impl From<BeaconStateError> for EpochProcessingError {
    /// Converts a `BeaconStateError` into an `EpochProcessingError`.
    fn from(e: BeaconStateError) -> EpochProcessingError {
        EpochProcessingError::BeaconStateError(e)
    }
}

impl From<ssz_types::Error> for EpochProcessingError {
    /// Converts an `ssz_types::Error` into an `EpochProcessingError`.
    fn from(e: ssz_types::Error) -> EpochProcessingError {
        EpochProcessingError::SszTypesError(e)
    }
}

impl From<safe_arith::ArithError> for EpochProcessingError {
    /// Converts a `safe_arith::ArithError` into an `EpochProcessingError`.
    fn from(e: safe_arith::ArithError) -> EpochProcessingError {
        EpochProcessingError::ArithError(e)
    }
}

impl From<milhouse::Error> for EpochProcessingError {
    /// Converts a `milhouse::Error` into an `EpochProcessingError`.
    fn from(e: milhouse::Error) -> Self {
        Self::MilhouseError(e)
    }
}

impl From<EpochCacheError> for EpochProcessingError {
    /// Converts an `EpochCacheError` into an `EpochProcessingError`.
    fn from(e: EpochCacheError) -> Self {
        EpochProcessingError::EpochCache(e)
    }
}

/// Errors that can occur during inclusion processing.
#[derive(Debug, PartialEq)]
pub enum InclusionError {
    /// The validator did not participate in an attestation in this period.
    NoAttestationsForValidator,
    /// An error related to the beacon state.
    BeaconStateError(BeaconStateError),
}

impl From<BeaconStateError> for InclusionError {
    /// Converts a `BeaconStateError` into an `InclusionError`.
    fn from(e: BeaconStateError) -> InclusionError {
        InclusionError::BeaconStateError(e)
    }
}
