//! Contains RPC handler implementations specific to tracing.

use reth_evm::ConfigureEvm;
use reth_rpc_eth_api::helpers::{LoadState, Trace};

use crate::EthApi;

/// Implementation of the `Trace` trait for the `EthApi` struct.
/// This trait provides methods related to tracing the Ethereum network.
impl<Provider, Pool, Network, EvmConfig> Trace for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: LoadState,
    EvmConfig: ConfigureEvm,
{
    /// Returns a reference to the EVM configuration.
    ///
    /// This method retrieves the EVM configuration from the inner configuration.
    ///
    /// # Returns
    ///
    /// A reference to an implementation of `ConfigureEvm`.
    #[inline]
    fn evm_config(&self) -> &impl ConfigureEvm {
        self.inner.evm_config()
    }
}
