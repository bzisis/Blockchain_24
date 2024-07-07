//! This module defines CLI arguments and their parsing logic for configuring parameters related to RPC state caching.
//! 
//! The `RpcStateCacheArgs` struct encapsulates options for:
//! - `max_blocks`: Maximum number of blocks to cache.
//! - `max_receipts`: Maximum number of receipts to cache.
//! - `max_envs`: Maximum number of bytes for cached environment data.
//! - `max_concurrent_db_requests`: Maximum number of concurrent database requests.
//! 
//! These parameters are used to tune the performance and behavior of the RPC server's state cache, ensuring efficient data retrieval and management.
//!
//! The `RpcStateCacheArgs` struct implements default values for these parameters, utilizing constants such as `DEFAULT_BLOCK_CACHE_MAX_LEN`, `DEFAULT_RECEIPT_CACHE_MAX_LEN`, `DEFAULT_ENV_CACHE_MAX_LEN`, and `DEFAULT_CONCURRENT_DB_REQUESTS`.
//!
//! Tests are included to validate the parsing and default behavior of `RpcStateCacheArgs`, ensuring correct configuration when parsed with various command-line arguments.
use clap::Args;
use reth_rpc_server_types::constants::cache::{
    DEFAULT_BLOCK_CACHE_MAX_LEN, DEFAULT_CONCURRENT_DB_REQUESTS, DEFAULT_ENV_CACHE_MAX_LEN,
    DEFAULT_RECEIPT_CACHE_MAX_LEN,
};

/// Parameters to configure RPC state cache.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(next_help_heading = "RPC State Cache")]
pub struct RpcStateCacheArgs {
    /// Max number of blocks in cache.
    #[arg(
        long = "rpc-cache.max-blocks",
        default_value_t = DEFAULT_BLOCK_CACHE_MAX_LEN,
    )]
    pub max_blocks: u32,

    /// Max number receipts in cache.
    #[arg(
        long = "rpc-cache.max-receipts",
        default_value_t = DEFAULT_RECEIPT_CACHE_MAX_LEN,
    )]
    pub max_receipts: u32,

    /// Max number of bytes for cached env data.
    #[arg(
        long = "rpc-cache.max-envs",
        default_value_t = DEFAULT_ENV_CACHE_MAX_LEN,
    )]
    pub max_envs: u32,

    /// Max number of concurrent database requests.
    #[arg(
        long = "rpc-cache.max-concurrent-db-requests",
        default_value_t = DEFAULT_CONCURRENT_DB_REQUESTS,
    )]
    pub max_concurrent_db_requests: usize,
}

impl Default for RpcStateCacheArgs {
    fn default() -> Self {
        Self {
            max_blocks: DEFAULT_BLOCK_CACHE_MAX_LEN,
            max_receipts: DEFAULT_RECEIPT_CACHE_MAX_LEN,
            max_envs: DEFAULT_ENV_CACHE_MAX_LEN,
            max_concurrent_db_requests: DEFAULT_CONCURRENT_DB_REQUESTS,
        }
    }
}
