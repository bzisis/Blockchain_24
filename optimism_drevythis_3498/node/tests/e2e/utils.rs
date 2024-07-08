use alloy_genesis::Genesis;
use reth::{
    rpc::types::engine::PayloadAttributes,
    tasks::TaskManager,
};
use reth_chainspec::{ChainSpecBuilder, BASE_MAINNET};
use reth_e2e_test_utils::{
    transaction::TransactionTestContext,
    wallet::Wallet,
    NodeHelperType,
};
use reth_node_optimism::{
    OptimismBuiltPayload,
    OptimismNode,
    OptimismPayloadBuilderAttributes,
};
use reth_payload_builder::EthPayloadBuilderAttributes;
use reth_primitives::{Address, B256};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Optimism Node Helper type
pub(crate) type OpNode = NodeHelperType<OptimismNode>;

/// Loads the genesis configuration and sets up the nodes for testing.
pub(crate) async fn setup(num_nodes: usize) -> eyre::Result<(Vec<OpNode>, TaskManager, Wallet)> {
    // Load genesis configuration from JSON file
    let genesis: Genesis = serde_json::from_str(include_str!("../assets/genesis.json"))?;

    // Set up nodes using test utilities with the specified number of nodes and genesis configuration
    let result = reth_e2e_test_utils::setup(
        num_nodes,
        Arc::new(
            ChainSpecBuilder::default()
                .chain(BASE_MAINNET.chain)
                .genesis(genesis)
                .ecotone_activated()
                .build(),
        ),
        false,
    )
    .await?;

    Ok(result)
}

/// Advances the chain by generating a sequence of payloads and returns them.
///
/// # Parameters
///
/// - `length`: The number of blocks to advance.
/// - `node`: Reference to the Optimism node.
/// - `wallet`: Shared reference to the wallet used for transactions.
///
/// # Returns
///
/// A vector of tuples containing generated payloads and their builder attributes.
pub(crate) async fn advance_chain(
    length: usize,
    node: &mut OpNode,
    wallet: Arc<Mutex<Wallet>>,
) -> eyre::Result<Vec<(OptimismBuiltPayload, OptimismPayloadBuilderAttributes)>> {
    // Advance the chain by generating payloads
    let result = node.advance(
        length as u64,
        |_| {
            let wallet = wallet.clone();
            Box::pin(async move {
                // Generate a transaction context for Optimism L1 block info
                let mut wallet = wallet.lock().await;
                let tx_fut = TransactionTestContext::optimism_l1_block_info_tx(
                    wallet.chain_id,
                    wallet.inner.clone(),
                    wallet.inner_nonce,
                );
                wallet.inner_nonce += 1;
                tx_fut.await
            })
        },
        optimism_payload_attributes,
    )
    .await?;

    Ok(result)
}

/// Constructs Optimism payload builder attributes based on provided timestamp.
///
/// # Parameters
///
/// - `timestamp`: Timestamp to be included in the payload attributes.
///
/// # Returns
///
/// Optimism payload builder attributes initialized with default values and the provided timestamp.
pub(crate) fn optimism_payload_attributes(timestamp: u64) -> OptimismPayloadBuilderAttributes {
    // Define payload attributes for the transaction
    let attributes = PayloadAttributes {
        timestamp,
        prev_randao: B256::ZERO,
        suggested_fee_recipient: Address::ZERO,
        withdrawals: Some(vec![]),
        parent_beacon_block_root: Some(B256::ZERO),
    };

    // Initialize Optimism payload builder attributes with default values and the specified timestamp
    OptimismPayloadBuilderAttributes {
        payload_attributes: EthPayloadBuilderAttributes::new(B256::ZERO, attributes),
        transactions: vec![],
        no_tx_pool: false,
        gas_limit: Some(30_000_000),
    }
}
