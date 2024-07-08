//! Optimism Consensus implementation.

// Documentation attributes for the crate. These include:
// - `html_logo_url`: URL for the HTML logo in documentation.
// - `html_favicon_url`: URL for the favicon in documentation.
// - `issue_tracker_base_url`: Base URL for the issue tracker.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
// Conditional compilation attributes for the Rust documentation system (docs.rs).
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
// Ensures the `optimism` feature is enabled to use this crate.
#![cfg(feature = "optimism")]

// Import necessary modules and traits from other crates.
use reth_chainspec::{ChainSpec, EthereumHardforks, OptimismHardforks};
use reth_consensus::{Consensus, ConsensusError, PostExecutionInput};
use reth_consensus_common::validation::{
    validate_against_parent_4844, validate_against_parent_eip1559_base_fee,
    validate_against_parent_hash_number, validate_against_parent_timestamp,
    validate_block_pre_execution, validate_header_base_fee, validate_header_extradata,
    validate_header_gas,
};
use reth_primitives::{
    BlockWithSenders, Header, SealedBlock, SealedHeader, EMPTY_OMMER_ROOT_HASH, U256,
};
use std::{sync::Arc, time::SystemTime};

// Module for post-execution validation.
mod validation;
pub use validation::validate_block_post_execution;

/// Struct implementing the Optimism consensus.
///
/// This struct provides basic checks as specified in the execution specs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimismBeaconConsensus {
    /// Chain configuration specifications.
    chain_spec: Arc<ChainSpec>,
}

impl OptimismBeaconConsensus {
    /// Creates a new instance of `OptimismBeaconConsensus`.
    ///
    /// # Panics
    ///
    /// Panics if the provided chain specification is not for Optimism.
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        // Ensure that the chain spec is for an Optimism chain.
        assert!(chain_spec.is_optimism(), "optimism consensus only valid for optimism chains");
        Self { chain_spec }
    }
}

impl Consensus for OptimismBeaconConsensus {
    /// Validates the header of a block.
    ///
    /// Validates the gas and base fee fields of the block header.
    fn validate_header(&self, header: &SealedHeader) -> Result<(), ConsensusError> {
        validate_header_gas(header)?;
        validate_header_base_fee(header, &self.chain_spec)
    }

    /// Validates the header against its parent.
    ///
    /// Ensures continuity and consistency with the parent block.
    fn validate_header_against_parent(
        &self,
        header: &SealedHeader,
        parent: &SealedHeader,
    ) -> Result<(), ConsensusError> {
        validate_against_parent_hash_number(header, parent)?;

        // Additional timestamp validation if Bedrock is active.
        if self.chain_spec.is_bedrock_active_at_block(header.number) {
            validate_against_parent_timestamp(header, parent)?;
        }

        validate_against_parent_eip1559_base_fee(header, parent, &self.chain_spec)?;

        // Validate the blob gas fields for this block if Cancun is active.
        if self.chain_spec.is_cancun_active_at_timestamp(header.timestamp) {
            validate_against_parent_4844(header, parent)?;
        }

        Ok(())
    }

    /// Validates the header with the total difficulty.
    ///
    /// Ensures the correctness of the block header based on the total difficulty.
    fn validate_header_with_total_difficulty(
        &self,
        header: &Header,
        _total_difficulty: U256,
    ) -> Result<(), ConsensusError> {
        // Determine if the Bedrock update has activated based on the block number.
        let is_post_merge = self.chain_spec.is_bedrock_active_at_block(header.number);

        if is_post_merge {
            // Post-merge checks.
            if header.nonce != 0 {
                return Err(ConsensusError::TheMergeNonceIsNotZero)
            }

            if header.ommers_hash != EMPTY_OMMER_ROOT_HASH {
                return Err(ConsensusError::TheMergeOmmerRootIsNotEmpty)
            }

            // Post-merge, validate header extradata for all networks.
            validate_header_extradata(header)?;

            // Note: MixHash is used instead of difficulty inside the EVM post-merge.
        } else {
            // Pre-merge check for future timestamps to prevent consensus issues.
            let present_timestamp =
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

            if header.exceeds_allowed_future_timestamp(present_timestamp) {
                return Err(ConsensusError::TimestampIsInFuture {
                    timestamp: header.timestamp,
                    present_timestamp,
                })
            }
        }

        Ok(())
    }

    /// Pre-execution validation of a block.
    ///
    /// Ensures the block meets the necessary conditions before execution.
    fn validate_block_pre_execution(&self, block: &SealedBlock) -> Result<(), ConsensusError> {
        validate_block_pre_execution(block, &self.chain_spec)
    }

    /// Post-execution validation of a block.
    ///
    /// Ensures the block is valid after execution, using the provided receipts.
    fn validate_block_post_execution(
        &self,
        block: &BlockWithSenders,
        input: PostExecutionInput<'_>,
    ) -> Result<(), ConsensusError> {
        validate_block_post_execution(block, &self.chain_spec, input.receipts)
    }
}
