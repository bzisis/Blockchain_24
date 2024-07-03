// Import necessary modules and components
use crate::{
    engine_api::EngineApiTestContext, network::NetworkTestContext, payload::PayloadTestContext,
    rpc::RpcTestContext, traits::PayloadEnvelopeExt,
};

use alloy_rpc_types::BlockNumberOrTag;
use eyre::Ok;
use futures_util::Future;
use reth::{
    api::{BuiltPayload, EngineTypes, FullNodeComponents, PayloadBuilderAttributes},
    builder::FullNode,
    payload::PayloadTypes,
    providers::{BlockReader, BlockReaderIdExt, CanonStateSubscriptions, StageCheckpointReader},
    rpc::types::engine::PayloadStatusEnum,
};
use reth_node_builder::NodeTypes;
use reth_primitives::{BlockHash, BlockNumber, Bytes, B256};
use reth_stages_types::StageId;
use std::{marker::PhantomData, pin::Pin};
use tokio_stream::StreamExt;

/// A helper struct to handle node actions
pub struct NodeTestContext<Node>
where
    Node: FullNodeComponents,
{
    pub inner: FullNode<Node>, // Full node instance
    pub payload: PayloadTestContext<Node::Engine>, // Payload test context
    pub network: NetworkTestContext, // Network test context
    pub engine_api: EngineApiTestContext<Node::Engine>, // Engine API test context
    pub rpc: RpcTestContext<Node>, // RPC test context
}

impl<Node> NodeTestContext<Node>
where
    Node: FullNodeComponents,
{
    /// Creates a new test node
    pub async fn new(node: FullNode<Node>) -> eyre::Result<Self> {
        let builder = node.payload_builder.clone(); // Clone the payload builder

        Ok(Self {
            inner: node.clone(), // Clone the full node
            payload: PayloadTestContext::new(builder).await?, // Create payload test context
            network: NetworkTestContext::new(node.network.clone()), // Create network test context
            engine_api: EngineApiTestContext {
                engine_api_client: node.auth_server_handle().http_client(), // Set up engine API client
                canonical_stream: node.provider.canonical_state_stream(), // Set up canonical state stream
                _marker: PhantomData::<Node::Engine>, // Phantom data for engine type
            },
            rpc: RpcTestContext { inner: node.rpc_registry }, // Set up RPC test context
        })
    }

    /// Establish a connection to the node
    pub async fn connect(&mut self, node: &mut NodeTestContext<Node>) {
        self.network.add_peer(node.network.record()).await; // Add peer to network
        node.network.next_session_established().await; // Wait for session establishment
        self.network.next_session_established().await; // Wait for session establishment
    }

    /// Advances the chain by `length` blocks.
    /// Returns the added chain as a Vec of block hashes.
    pub async fn advance(
        &mut self,
        length: u64, // Number of blocks to advance
        tx_generator: impl Fn(u64) -> Pin<Box<dyn Future<Output = Bytes>>>, // Transaction generator
        attributes_generator: impl Fn(u64) -> <Node::Engine as PayloadTypes>::PayloadBuilderAttributes
            + Copy, // Payload attributes generator
    ) -> eyre::Result<
        Vec<(
            <Node::Engine as PayloadTypes>::BuiltPayload, // Built payload
            <Node::Engine as PayloadTypes>::PayloadBuilderAttributes, // Payload builder attributes
        )>,
    >
    where
        <Node::Engine as EngineTypes>::ExecutionPayloadV3:
            From<<Node::Engine as PayloadTypes>::BuiltPayload> + PayloadEnvelopeExt,
    {
        let mut chain = Vec::with_capacity(length as usize); // Initialize chain vector
        for i in 0..length {
            let raw_tx = tx_generator(i).await; // Generate raw transaction
            let tx_hash = self.rpc.inject_tx(raw_tx).await?; // Inject transaction and get hash
            let (payload, eth_attr) = self.advance_block(vec![], attributes_generator).await?; // Advance block
            let block_hash = payload.block().hash(); // Get block hash
            let block_number = payload.block().number; // Get block number
            self.assert_new_block(tx_hash, block_hash, block_number).await?; // Assert new block
            chain.push((payload, eth_attr)); // Add payload and attributes to chain
        }
        Ok(chain)
    }

    /// Creates a new payload from given attributes generator
    /// expects a payload attribute event and waits until the payload is built.
    ///
    /// It triggers the resolve payload via engine API and expects the built payload event.
    pub async fn new_payload(
        &mut self,
        attributes_generator: impl Fn(u64) -> <Node::Engine as PayloadTypes>::PayloadBuilderAttributes,
    ) -> eyre::Result<(
        <<Node as NodeTypes>::Engine as PayloadTypes>::BuiltPayload, // Built payload
        <<Node as NodeTypes>::Engine as PayloadTypes>::PayloadBuilderAttributes, // Payload builder attributes
    )>
    where
        <Node::Engine as EngineTypes>::ExecutionPayloadV3:
            From<<Node::Engine as PayloadTypes>::BuiltPayload> + PayloadEnvelopeExt,
    {
        // Trigger new payload building by draining the pool
        let eth_attr = self.payload.new_payload(attributes_generator).await.unwrap();
        // Expect a payload attribute event
        self.payload.expect_attr_event(eth_attr.clone()).await?;
        // Wait for the payload builder to finish building
        self.payload.wait_for_built_payload(eth_attr.payload_id()).await;
        // Trigger resolve payload via engine API
        self.engine_api.get_payload_v3_value(eth_attr.payload_id()).await?;
        // Ensure we're also receiving the built payload as an event
        Ok((self.payload.expect_built_payload().await?, eth_attr))
    }

    /// Advances the node forward by one block
    pub async fn advance_block(
        &mut self,
        versioned_hashes: Vec<B256>, // Versioned hashes
        attributes_generator: impl Fn(u64) -> <Node::Engine as PayloadTypes>::PayloadBuilderAttributes,
    ) -> eyre::Result<(
        <Node::Engine as PayloadTypes>::BuiltPayload, // Built payload
        <<Node as NodeTypes>::Engine as PayloadTypes>::PayloadBuilderAttributes, // Payload builder attributes
    )>
    where
        <Node::Engine as EngineTypes>::ExecutionPayloadV3:
            From<<Node::Engine as PayloadTypes>::BuiltPayload> + PayloadEnvelopeExt,
    {
        let (payload, eth_attr) = self.new_payload(attributes_generator).await?; // Create new payload

        let block_hash = self
            .engine_api
            .submit_payload(
                payload.clone(), // Clone payload
                eth_attr.clone(), // Clone attributes
                PayloadStatusEnum::Valid, // Set payload status to valid
                versioned_hashes, // Set versioned hashes
            )
            .await?;

        // Trigger forkchoice update via engine API to commit the block to the blockchain
        self.engine_api.update_forkchoice(block_hash, block_hash).await?;

        Ok((payload, eth_attr))
    }

    /// Waits for block to be available on the node.
    pub async fn wait_block(
        &self,
        number: BlockNumber, // Block number to wait for
        expected_block_hash: BlockHash, // Expected block hash
        wait_finish_checkpoint: bool, // Flag to wait for finish checkpoint
    ) -> eyre::Result<()> {
        let mut check = !wait_finish_checkpoint; // Set initial check flag
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await; // Sleep for 20 milliseconds

            if !check && wait_finish_checkpoint {
                if let Some(checkpoint) =
                    self.inner.provider.get_stage_checkpoint(StageId::Finish)?
                {
                    if checkpoint.block_number >= number {
                        check = true // Set check flag if checkpoint block number is greater or equal to the expected number
                    }
                }
            }

            if check {
                if let Some(latest_block) = self.inner.provider.block_by_number(number)? {
                    assert_eq!(latest_block.hash_slow(), expected_block_hash); // Assert block hash matches the expected hash
                    break
                }
                if wait_finish_checkpoint {
                    panic!("Finish checkpoint matches, but could not fetch block."); // Panic if block fetch fails
                }
            }
        }
        Ok(())
    }

    /// Waits for the block to be unwound
    pub async fn wait_unwind(&self, number: BlockNumber) -> eyre::Result<()> {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await; // Sleep for 10 milliseconds
            if let Some(checkpoint) = self.inner.provider.get_stage_checkpoint(StageId::Headers)? {
                if checkpoint.block_number == number {
                    break // Break loop if checkpoint block number matches the expected number
                }
            }
        }
        Ok(())
    }

    /// Asserts that a new block has been added to the blockchain
    /// and the transaction has been included in the block.
    ///
    /// Does NOT work for pipeline since there's no stream notification!
    pub async fn assert_new_block(
        &mut self,
        tip_tx_hash: B256, // Transaction hash of the tip
        block_hash: B256, // Block hash
        block_number: BlockNumber, // Block number
    ) -> eyre::Result<()> {
        // Get the head block from notifications stream and verify the transaction has been included
        let head = self.engine_api.canonical_stream.next().await.unwrap();