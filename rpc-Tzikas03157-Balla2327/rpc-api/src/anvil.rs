use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use reth_primitives::{Address, Bytes, B256, U256};
use reth_rpc_types::{
    anvil::{Forking, Metadata, MineOptions, NodeInfo},
    Block,
};

/// Anvil rpc interface.
/// Provides custom methods for interacting with the Anvil node.
/// See: <https://book.getfoundry.sh/reference/anvil/#custom-methods>
#[cfg_attr(not(feature = "client"), rpc(server, namespace = "anvil"))]
#[cfg_attr(feature = "client", rpc(server, client, namespace = "anvil"))]
pub trait AnvilApi {
    /// Sends transactions impersonating specific account and contract addresses.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to impersonate.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "impersonateAccount")]
    async fn anvil_impersonate_account(&self, address: Address) -> RpcResult<()>;

    /// Stops impersonating an account if previously set with `anvil_impersonateAccount`.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to stop impersonating.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "stopImpersonatingAccount")]
    async fn anvil_stop_impersonating_account(&self, address: Address) -> RpcResult<()>;

    /// If set to true will make every account impersonated.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Boolean to enable or disable auto impersonation.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "autoImpersonateAccount")]
    async fn anvil_auto_impersonate_account(&self, enabled: bool) -> RpcResult<()>;

    /// Returns `true` if auto mining is enabled, and `false` otherwise.
    ///
    /// # Returns
    ///
    /// A `RpcResult` with the current automine status.
    #[method(name = "getAutomine")]
    async fn anvil_get_automine(&self) -> RpcResult<bool>;

    /// Mines a series of blocks.
    ///
    /// # Arguments
    ///
    /// * `blocks` - The number of blocks to mine.
    /// * `interval` - The interval between blocks in seconds.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "mine")]
    async fn anvil_mine(&self, blocks: Option<U256>, interval: Option<U256>) -> RpcResult<()>;

    /// Enables or disables automatic mining of new blocks with each new transaction.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Boolean to enable or disable automine.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setAutomine")]
    async fn anvil_set_automine(&self, enabled: bool) -> RpcResult<()>;

    /// Sets the mining behavior to interval with the given interval in seconds.
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval for mining in seconds.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setIntervalMining")]
    async fn anvil_set_interval_mining(&self, interval: u64) -> RpcResult<()>;

    /// Removes transactions from the pool.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The hash of the transaction to remove.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing an optional `B256` if the transaction was successfully removed.
    #[method(name = "anvil_dropTransaction")]
    async fn anvil_drop_transaction(&self, tx_hash: B256) -> RpcResult<Option<B256>>;

    /// Resets the fork to a fresh forked state, and optionally update the fork config.
    ///
    /// # Arguments
    ///
    /// * `fork` - An optional `Forking` configuration.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "reset")]
    async fn anvil_reset(&self, fork: Option<Forking>) -> RpcResult<()>;

    /// Sets the backend RPC URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The new RPC URL.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setRpcUrl")]
    async fn anvil_set_rpc_url(&self, url: String) -> RpcResult<()>;

    /// Modifies the balance of an account.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the account.
    /// * `balance` - The new balance.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setBalance")]
    async fn anvil_set_balance(&self, address: Address, balance: U256) -> RpcResult<()>;

    /// Sets the code of a contract.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract.
    /// * `code` - The new code in bytes.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setCode")]
    async fn anvil_set_code(&self, address: Address, code: Bytes) -> RpcResult<()>;

    /// Sets the nonce of an address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address.
    /// * `nonce` - The new nonce.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setNonce")]
    async fn anvil_set_nonce(&self, address: Address, nonce: U256) -> RpcResult<()>;

    /// Writes a single slot of the account's storage.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the account.
    /// * `slot` - The storage slot.
    /// * `value` - The value to write.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setStorageAt")]
    async fn anvil_set_storage_at(
        &self,
        address: Address,
        slot: U256,
        value: B256,
    ) -> RpcResult<bool>;

    /// Sets the coinbase address.
    ///
    /// # Arguments
    ///
    /// * `address` - The new coinbase address.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setCoinbase")]
    async fn anvil_set_coinbase(&self, address: Address) -> RpcResult<()>;

    /// Sets the chain ID.
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The new chain ID.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setChainId")]
    async fn anvil_set_chain_id(&self, chain_id: u64) -> RpcResult<()>;

    /// Enables or disables logging.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Boolean to enable or disable logging.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setLoggingEnabled")]
    async fn anvil_set_logging_enabled(&self, enabled: bool) -> RpcResult<()>;

    /// Sets the minimum gas price for the node.
    ///
    /// # Arguments
    ///
    /// * `gas_price` - The new minimum gas price.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setMinGasPrice")]
    async fn anvil_set_min_gas_price(&self, gas_price: U256) -> RpcResult<()>;

    /// Sets the base fee of the next block.
    ///
    /// # Arguments
    ///
    /// * `base_fee` - The new base fee.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setNextBlockBaseFeePerGas")]
    async fn anvil_set_next_block_base_fee_per_gas(&self, base_fee: U256) -> RpcResult<()>;

    /// Sets the time for the next block.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The new timestamp.
    ///
    /// # Returns
    ///
    /// A `RpcResult` with the new timestamp.
    #[method(name = "setTime")]
    async fn anvil_set_time(&self, timestamp: u64) -> RpcResult<u64>;

    /// Creates a buffer that represents all state on the chain, which can be loaded to separate
    /// process by calling `anvil_loadState`.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing the state as `Bytes`.
    #[method(name = "dumpState")]
    async fn anvil_dump_state(&self) -> RpcResult<Bytes>;

    /// Appends chain state buffer to the current chain. Will overwrite any conflicting addresses or
    /// storage.
    ///
    /// # Arguments
    ///
    /// * `state` - The state to load as `Bytes`.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "loadState")]
    async fn anvil_load_state(&self, state: Bytes) -> RpcResult<bool>;

    /// Retrieves the Anvil node configuration parameters.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing the `NodeInfo`.
    #[method(name = "nodeInfo")]
    async fn anvil_node_info(&self) -> RpcResult<NodeInfo>;

    /// Retrieves metadata about the Anvil instance.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing the `Metadata`.
    #[method(name = "metadata")]
    async fn anvil_metadata(&self) -> RpcResult<Metadata>;

    /// Snapshots the state of the blockchain at the current block.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing the snapshot ID as `U256`.
    #[method(name = "snapshot")]
    async fn anvil_snapshot(&self) -> RpcResult<U256>;

    /// Reverts the state of the blockchain to a previous snapshot.
    ///
    /// # Arguments
    ///
    /// * `id` - The snapshot ID to revert to.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "revert")]
    async fn anvil_revert(&self, id: U256) -> RpcResult<bool>;

    /// Jumps forward in time by the given amount of time in seconds.
    ///
    /// # Arguments
    ///
    /// * `seconds` - The amount of time to jump forward in seconds.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing the new timestamp.
    #[method(name = "increaseTime")]
    async fn anvil_increase_time(&self, seconds: U256) -> RpcResult<i64>;

    /// Sets the exact timestamp for the next block.
    ///
    /// # Arguments
    ///
    /// * `seconds` - The new timestamp for the next block.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setNextBlockTimestamp")]
    async fn anvil_set_next_block_timestamp(&self, seconds: u64) -> RpcResult<()>;

    /// Sets the gas limit for the next block.
    ///
    /// # Arguments
    ///
    /// * `gas_limit` - The new gas limit.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setBlockGasLimit")]
    async fn anvil_set_block_gas_limit(&self, gas_limit: U256) -> RpcResult<bool>;

    /// Sets an interval for the block timestamp.
    ///
    /// # Arguments
    ///
    /// * `seconds` - The interval in seconds.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "setBlockTimestampInterval")]
    async fn anvil_set_block_timestamp_interval(&self, seconds: u64) -> RpcResult<()>;

    /// Removes the interval for the block timestamp.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "removeBlockTimestampInterval")]
    async fn anvil_remove_block_timestamp_interval(&self) -> RpcResult<bool>;

    /// Mines blocks instantly and returns the mined blocks.
    ///
    /// This will mine the blocks regardless of the configured mining mode.
    ///
    /// # Arguments
    ///
    /// * `opts` - The mining options.
    ///
    /// # Returns
    ///
    /// A `RpcResult` containing a vector of mined `Block`s.
    ///
    /// **Note**: This behaves exactly as `evm_mine` but returns different output. For
    /// compatibility reasons, this is a separate call since `evm_mine` is not an anvil original.
    #[method(name = "mine_detailed")]
    async fn anvil_mine_detailed(&self, opts: Option<MineOptions>) -> RpcResult<Vec<Block>>;

    /// Turns on call traces for transactions that are returned to the user when they execute a
    /// transaction (instead of just txhash/receipt).
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "enableTraces")]
    async fn anvil_enable_traces(&self) -> RpcResult<()>;

    /// Removes all transactions for the specified address from the transaction pool.
    ///
    /// # Arguments
    ///
    /// * `address` - The address whose transactions should be removed.
    ///
    /// # Returns
    ///
    /// A `RpcResult` indicating success or failure.
    #[method(name = "removePoolTransactions")]
    async fn anvil_remove_pool_transactions(&self, address: Address) -> RpcResult<()>;
}
