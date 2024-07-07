// Clippy lint set-up (disabled in tests)
#![cfg_attr(
    not(test),
    deny(
        clippy::arithmetic_side_effects,
        clippy::disallowed_methods,
        clippy::indexing_slicing,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::let_underscore_must_use
    )
)]

/// This module contains various macros used throughout the project.
#[macro_use]
mod macros;

/// This module contains metrics related functionality.
mod metrics;

/// Public module `all_caches` provides access to caches related to the system.
pub mod all_caches;

/// Public module `block_replayer` provides functionality for replaying blocks.
pub mod block_replayer;

/// Public module `common` contains common utilities and types used across the project.
pub mod common;

/// Public module `consensus_context` provides context information for consensus algorithms.
pub mod consensus_context;

/// Public module `epoch_cache` provides caching mechanisms related to epochs.
pub mod epoch_cache;

/// Public module `genesis` contains functions and utilities related to genesis state.
pub mod genesis;

/// Public module `per_block_processing` provides logic for processing blocks.
pub mod per_block_processing;

/// Public module `per_epoch_processing` provides logic for epoch-level processing.
pub mod per_epoch_processing;

/// Public module `per_slot_processing` provides logic for slot-level processing.
pub mod per_slot_processing;

/// Public module `state_advance` provides functionality for advancing the state.
pub mod state_advance;

/// Public module `upgrade` contains upgrade-related functionality.
pub mod upgrade;

/// Public module `verify_operation` provides verification logic for operations.
pub mod verify_operation;

/// Re-export of `AllCaches` from `all_caches` module.
pub use all_caches::AllCaches;

/// Re-export of `BlockReplayError` and `BlockReplayer` from `block_replayer` module.
pub use block_replayer::{BlockReplayError, BlockReplayer};

/// Re-export of `ConsensusContext` and `ContextError` from `consensus_context` module.
pub use consensus_context::{ConsensusContext, ContextError};

/// Re-export of several functions and utilities related to genesis state from `genesis` module.
pub use genesis::{
    eth2_genesis_time, initialize_beacon_state_from_eth1, is_valid_genesis_state,
    process_activations,
};

/// Re-export of types and utilities related to per-block processing from `per_block_processing` module.
pub use per_block_processing::{
    block_signature_verifier, errors::BlockProcessingError, per_block_processing, signature_sets,
    BlockSignatureStrategy, BlockSignatureVerifier, VerifyBlockRoot, VerifySignatures,
};

/// Re-export of functions and utilities related to per-epoch processing from `per_epoch_processing` module.
pub use per_epoch_processing::{
    errors::EpochProcessingError, process_epoch as per_epoch_processing,
};

/// Re-export of functions and utilities related to per-slot processing from `per_slot_processing` module.
pub use per_slot_processing::{per_slot_processing, Error as SlotProcessingError};

/// Re-export of types related to epoch caching from `types` module.
pub use types::{EpochCache, EpochCacheError, EpochCacheKey};
