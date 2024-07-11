//! Contains RPC handler implementations specific to endpoints that call/execute within evm.

use reth_evm::ConfigureEvm;
use reth_rpc_eth_api::helpers::{Call, EthCall, LoadPendingBlock, LoadState, SpawnBlocking};

use crate::EthApi;

/// Implements `EthCall` trait for `EthApi`
/// This trait provides methods for handling Ethereum call endpoints.
impl<Provider, Pool, Network, EvmConfig> EthCall for EthApi<Provider, Pool, Network, EvmConfig> where
    Self: Call + LoadPendingBlock
{
    // No additional methods needed, `EthCall` is implemented by combining `Call` and `LoadPendingBlock` traits
}

/// Implements `Call` trait for `EthApi`
/// This trait provides methods for calling/executing within EVM.
impl<Provider, Pool, Network, EvmConfig> Call for EthApi<Provider, Pool, Network, EvmConfig>
where
    Self: LoadState + SpawnBlocking,
    EvmConfig: ConfigureEvm,
{
    /// Returns the gas limit for calls.
    /// This method is used to enforce a gas limit on EVM calls to prevent excessive gas usage.
    #[inline]
    fn call_gas_limit(&self) -> u64 {
        self.inner.gas_cap()
    }

    /// Returns a reference to the EVM configuration.
    /// The EVM configuration contains settings and parameters for the Ethereum Virtual Machine.
    #[inline]
    fn evm_config(&self) -> &impl ConfigureEvm {
        self.inner.evm_config()
    }
}
