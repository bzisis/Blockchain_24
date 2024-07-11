use reth_chainspec::ChainInfo;
use reth_errors::{RethError, RethResult};
use reth_evm::ConfigureEvm;
use reth_network_api::NetworkInfo;
use reth_primitives::{Address, U256, U64};
use reth_provider::{BlockReaderIdExt, ChainSpecProvider, EvmEnvProvider, StateProviderFactory};
use reth_rpc_eth_api::helpers::EthApiSpec;
use reth_rpc_types::{SyncInfo, SyncStatus};
use reth_transaction_pool::TransactionPool;

use crate::EthApi;

/// Implementation of the `EthApiSpec` trait for the `EthApi` struct.
/// This trait provides a specification for Ethereum API, including methods for protocol version,
/// chain ID, chain info, accounts, syncing status, and sync status.
impl<Provider, Pool, Network, EvmConfig> EthApiSpec for EthApi<Provider, Pool, Network, EvmConfig>
where
    Pool: TransactionPool + 'static,
    Provider:
        BlockReaderIdExt + ChainSpecProvider + StateProviderFactory + EvmEnvProvider + 'static,
    Network: NetworkInfo + 'static,
    EvmConfig: ConfigureEvm,
{
    /// Returns the current Ethereum protocol version.
    ///
    /// This method returns a `U64` representing the protocol version.
    /// The protocol version is retrieved from the network status.
    ///
    /// # Returns
    ///
    /// A `RethResult<U64>` containing the protocol version.
    async fn protocol_version(&self) -> RethResult<U64> {
        // Fetch the network status and extract the protocol version.
        let status = self.network().network_status().await.map_err(RethError::other)?;
        Ok(U64::from(status.protocol_version))
    }

    /// Returns the chain ID.
    ///
    /// This method retrieves the chain ID from the network and returns it as a `U64`.
    ///
    /// # Returns
    ///
    /// A `U64` containing the chain ID.
    fn chain_id(&self) -> U64 {
        U64::from(self.network().chain_id())
    }

    /// Returns the current chain information.
    ///
    /// This method retrieves the chain info from the provider.
    ///
    /// # Returns
    ///
    /// A `RethResult<ChainInfo>` containing the chain information.
    fn chain_info(&self) -> RethResult<ChainInfo> {
        // Fetch the chain info from the provider.
        Ok(self.provider().chain_info()?)
    }

    /// Returns a list of addresses managed by the signer.
    ///
    /// This method retrieves the accounts from the signers and collects them into a vector.
    ///
    /// # Returns
    ///
    /// A `Vec<Address>` containing the addresses.
    fn accounts(&self) -> Vec<Address> {
        self.inner.signers().read().iter().flat_map(|s| s.accounts()).collect()
    }

    /// Checks if the network is currently syncing.
    ///
    /// This method returns a boolean indicating the syncing status of the network.
    ///
    /// # Returns
    ///
    /// `true` if the network is syncing, `false` otherwise.
    fn is_syncing(&self) -> bool {
        self.network().is_syncing()
    }

    /// Returns the sync status of the network.
    ///
    /// This method returns a `SyncStatus` indicating the current sync status of the network.
    ///
    /// # Returns
    ///
    /// A `RethResult<SyncStatus>` containing the sync status.
    fn sync_status(&self) -> RethResult<SyncStatus> {
        // Determine the sync status based on whether the network is currently syncing.
        let status = if self.is_syncing() {
            // If syncing, fetch the current block number and create a SyncStatus::Info.
            let current_block = U256::from(
                self.provider().chain_info().map(|info| info.best_number).unwrap_or_default(),
            );
            SyncStatus::Info(SyncInfo {
                starting_block: self.inner.starting_block(),
                current_block,
                highest_block: current_block,
                warp_chunks_amount: None,
                warp_chunks_processed: None,
            })
        } else {
            // If not syncing, return SyncStatus::None.
            SyncStatus::None
        };
        Ok(status)
    }
}
