mod error;
pub mod fork_choice_test_definition;
mod justified_balances;
mod proto_array;
mod proto_array_fork_choice;
mod ssz_container;

/// Re-export of `JustifiedBalances` from `justified_balances` module.
pub use crate::justified_balances::JustifiedBalances;

/// Re-export of `calculate_committee_fraction` and `InvalidationOperation` from `proto_array` module.
pub use crate::proto_array::{calculate_committee_fraction, InvalidationOperation};

/// Re-export of various types related to `ProtoArrayForkChoice` from `proto_array_fork_choice` module.
pub use crate::proto_array_fork_choice::{
    Block, DisallowedReOrgOffsets, DoNotReOrg, ExecutionStatus, ProposerHeadError,
    ProposerHeadInfo, ProtoArrayForkChoice, ReOrgThreshold,
};

/// Re-export of `Error` from the `error` module.
pub use error::Error;

/// Submodule `core` containing essential types for internal use.
pub mod core {
    /// Re-export of `ProposerBoost`, `ProtoArray`, and `ProtoNode` from `proto_array` module.
    pub use super::proto_array::{ProposerBoost, ProtoArray, ProtoNode};

    /// Re-export of `VoteTracker` from `proto_array_fork_choice` module.
    pub use super::proto_array_fork_choice::VoteTracker;

    /// Re-export of `SszContainer`, `SszContainerV16`, and `SszContainerV17` from `ssz_container` module.
    pub use super::ssz_container::{SszContainer, SszContainerV16, SszContainerV17};
}
