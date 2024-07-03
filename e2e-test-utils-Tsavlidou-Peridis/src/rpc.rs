// Import necessary modules and components
use alloy_consensus::TxEnvelope; // Transaction envelope type
use alloy_network::eip2718::Decodable2718; // Decoding for EIP-2718 transactions
use reth::{
    builder::{rpc::RpcRegistry, FullNodeComponents}, // Components for building full nodes and RPC registry
    rpc::{
        api::{eth::helpers::EthTransactions, DebugApiServer}, // Ethereum RPC APIs
        server_types::eth::EthResult, // Result type for Ethereum RPC
    },
};
use reth_primitives::{Bytes, B256}; // Primitive types for bytes and 256-bit hashes

// Define a struct for RPC test context
pub struct RpcTestContext<Node: FullNodeComponents> {
    pub inner: RpcRegistry<Node>, // Inner field for the RPC registry
}

impl<Node: FullNodeComponents> RpcTestContext<Node> {
    /// Injects a raw transaction into the node tx pool via RPC server
    pub async fn inject_tx(&mut self, raw_tx: Bytes) -> EthResult<B256> {
        // Get the Ethereum API from the RPC registry
        let eth_api = self.inner.eth_api();
        // Send the raw transaction and await the result
        eth_api.send_raw_transaction(raw_tx).await
    }

    /// Retrieves a transaction envelope by its hash
    pub async fn envelope_by_hash(&mut self, hash: B256) -> eyre::Result<TxEnvelope> {
        // Get the raw transaction from the debug API by its hash
        let tx = self.inner.debug_api().raw_transaction(hash).await?.unwrap();
        // Convert the transaction to a byte vector
        let tx = tx.to_vec();
        // Decode the transaction envelope using EIP-2718 decoding and return it
        Ok(TxEnvelope::decode_2718(&mut tx.as_ref()).unwrap())
    }
}