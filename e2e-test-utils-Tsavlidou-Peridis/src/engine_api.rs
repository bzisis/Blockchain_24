use crate::traits::PayloadEnvelopeExt; // Importing the PayloadEnvelopeExt trait from the traits crate
use jsonrpsee::{
    core::client::ClientT, // Importing the client traits for JSON-RPC communication
    http_client::{transport::HttpBackend, HttpClient}, // Importing the HTTP backend and client for JSON-RPC communication
};
use reth::{
    api::{EngineTypes, PayloadBuilderAttributes}, // Importing the EngineTypes and PayloadBuilderAttributes from the reth API
    providers::CanonStateNotificationStream, // Importing the CanonStateNotificationStream provider
    rpc::{
        api::EngineApiClient, // Importing the EngineApiClient for API calls
        types::engine::{ForkchoiceState, PayloadStatusEnum}, // Importing types related to the engine such as ForkchoiceState and PayloadStatusEnum
    },
};
// Importing crates
use reth_payload_builder::PayloadId;    
use reth_primitives::B256;              
use reth_rpc_layer::AuthClientService;  
use std::marker::PhantomData;           

/// Helper for engine api operations
pub struct EngineApiTestContext<E> {
    pub canonical_stream: CanonStateNotificationStream, // Stream for canonical state notifications
    pub engine_api_client: HttpClient<AuthClientService<HttpBackend>>, // HTTP client for engine API calls
    pub _marker: PhantomData<E>, // Phantom data for generic type E
}

impl<E: EngineTypes + 'static> EngineApiTestContext<E> {
    /// Retrieves a v3 payload from the engine API
    pub async fn get_payload_v3(
        &self,
        payload_id: PayloadId,
    ) -> eyre::Result<E::ExecutionPayloadV3> {
        // Use the engine API client to get the v3 payload
        Ok(EngineApiClient::<E>::get_payload_v3(&self.engine_api_client, payload_id).await?)
    }

    /// Retrieves a v3 payload from the engine API as a serde JSON value
    pub async fn get_payload_v3_value(
        &self,
        payload_id: PayloadId,
    ) -> eyre::Result<serde_json::Value> {
        // Use the engine API client to get the v3 payload and deserialize it to serde_json::Value
        Ok(self.engine_api_client.request("engine_getPayloadV3", (payload_id,)).await?)
    }

    /// Submits a payload to the engine API
    pub async fn submit_payload(
        &self,
        payload: E::BuiltPayload, // Payload to submit
        payload_builder_attributes: E::PayloadBuilderAttributes, // Attributes used for building the payload
        expected_status: PayloadStatusEnum, // Expected status of the payload submission
        versioned_hashes: Vec<B256>, // Versioned hashes related to the payload
    ) -> eyre::Result<B256>
    where
        E::ExecutionPayloadV3: From<E::BuiltPayload> + PayloadEnvelopeExt,
    {
        // setup payload for submission
        let envelope_v3: <E as EngineTypes>::ExecutionPayloadV3 = payload.into();

        // submit payload to engine api
        let submission = EngineApiClient::<E>::new_payload_v3(
            &self.engine_api_client,
            envelope_v3.execution_payload(),
            versioned_hashes,
            payload_builder_attributes.parent_beacon_block_root().unwrap(),
        )
        .await?;

        // Ensure the submission status matches the expected status
        assert_eq!(submission.status, expected_status);

        // Return the latest valid hash or a default value if none
        Ok(submission.latest_valid_hash.unwrap_or_default())
    }

    /// Sends a forkchoice update to the engine API
    pub async fn update_forkchoice(&self, current_head: B256, new_head: B256) -> eyre::Result<()> {
        EngineApiClient::<E>::fork_choice_updated_v2(
            &self.engine_api_client,
            ForkchoiceState {
                head_block_hash: new_head,
                safe_block_hash: current_head,
                finalized_block_hash: current_head,
            },
            None,
        )
        .await?;
        Ok(())
    }

    /// Sends a forkchoice update to the engine API with a zero finalized hash
    pub async fn update_optimistic_forkchoice(&self, hash: B256) -> eyre::Result<()> {
        EngineApiClient::<E>::fork_choice_updated_v2(
            &self.engine_api_client,
            ForkchoiceState {
                head_block_hash: hash,
                safe_block_hash: B256::ZERO,
                finalized_block_hash: B256::ZERO,
            },
            None,
        )
        .await?;

        Ok(())
    }
}