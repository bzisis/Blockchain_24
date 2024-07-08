//! Error types for the Optimism EVM module.

use reth_evm::execute::BlockExecutionError;

/// Optimism Block Executor Errors
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum OptimismBlockExecutionError {
    /// Error when trying to parse L1 block info
    #[error("could not get L1 block info from L2 block: {message:?}")]
    L1BlockInfoError {
        /// The inner error message
        message: String,  // Describes the error encountered when parsing L1 block info from an L2 block
    },
    /// Thrown when force deploy of create2deployer code fails.
    #[error("failed to force create2deployer account code")]
    ForceCreate2DeployerFail,  // Error when the forced deployment of the create2deployer account code fails
    /// Thrown when a blob transaction is included in a sequencer's block.
    #[error("blob transaction included in sequencer block")]
    BlobTransactionRejected,  // Error when a blob transaction is incorrectly included in a sequencer's block
    /// Thrown when a database account could not be loaded.
    #[error("failed to load account {0}")]
    AccountLoadFailed(reth_primitives::Address),  // Error when a database account could not be loaded, includes the account address
}

// Implementing conversion from `OptimismBlockExecutionError` to `BlockExecutionError`
impl From<OptimismBlockExecutionError> for BlockExecutionError {
    fn from(err: OptimismBlockExecutionError) -> Self {
        Self::other(err)  // Converts the Optimism-specific error into a generic block execution error
    }
}
