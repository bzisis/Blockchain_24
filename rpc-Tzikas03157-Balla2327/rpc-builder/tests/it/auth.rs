//! Auth server tests
//!
//! This module contains tests for the authentication server, verifying the functionality of
//! various engine calls through HTTP and WebSocket clients.

use crate::utils::launch_auth;
use jsonrpsee::core::client::{ClientT, SubscriptionClientT};
use reth_ethereum_engine_primitives::EthEngineTypes;
use reth_primitives::{Block, U64};
use reth_rpc_api::clients::EngineApiClient;
use reth_rpc_layer::JwtSecret;
use reth_rpc_types::engine::{ForkchoiceState, PayloadId, TransitionConfiguration};
use reth_rpc_types_compat::engine::payload::{
    block_to_payload_v1, convert_block_to_payload_input_v2,
};

/// Tests basic engine calls using a provided client.
///
/// This function tests various engine API calls to ensure they are functioning correctly.
/// It performs the following actions:
/// - Sends a new payload (v1 and v2).
/// - Updates fork choice (v1).
/// - Retrieves a payload (v1 and v2).
/// - Retrieves payload bodies by hash and by range (v1).
/// - Exchanges transition configuration.
/// - Exchanges capabilities.
///
/// # Arguments
///
/// * `client` - A reference to the client implementing necessary traits for engine API calls.
#[allow(unused_must_use)]
async fn test_basic_engine_calls<C>(client: &C)
where
    C: ClientT + SubscriptionClientT + Sync + EngineApiClient<EthEngineTypes>,
{
    let block = Block::default().seal_slow();
    EngineApiClient::new_payload_v1(client, block_to_payload_v1(block.clone())).await;
    EngineApiClient::new_payload_v2(client, convert_block_to_payload_input_v2(block)).await;
    EngineApiClient::fork_choice_updated_v1(client, ForkchoiceState::default(), None).await;
    EngineApiClient::get_payload_v1(client, PayloadId::new([0, 0, 0, 0, 0, 0, 0, 0])).await;
    EngineApiClient::get_payload_v2(client, PayloadId::new([0, 0, 0, 0, 0, 0, 0, 0])).await;
    EngineApiClient::get_payload_bodies_by_hash_v1(client, vec![]).await;
    EngineApiClient::get_payload_bodies_by_range_v1(client, U64::ZERO, U64::from(1u64)).await;
    EngineApiClient::exchange_transition_configuration(client, TransitionConfiguration::default())
        .await;
    EngineApiClient::exchange_capabilities(client, vec![]).await;
}

/// Tests the authentication server endpoints over HTTP.
///
/// This test initializes tracing, generates a random JWT secret, launches the auth server, and
/// performs basic engine API calls using an HTTP client.
#[tokio::test(flavor = "multi_thread")]
async fn test_auth_endpoints_http() {
    reth_tracing::init_test_tracing();
    let secret = JwtSecret::random();
    let handle = launch_auth(secret).await;
    let client = handle.http_client();
    test_basic_engine_calls(&client).await
}

/// Tests the authentication server endpoints over WebSocket.
///
/// This test initializes tracing, generates a random JWT secret, launches the auth server, and
/// performs basic engine API calls using a WebSocket client.
#[tokio::test(flavor = "multi_thread")]
async fn test_auth_endpoints_ws() {
    reth_tracing::init_test_tracing();
    let secret = JwtSecret::random();
    let handle = launch_auth(secret).await;
    let client = handle.ws_client().await;
    test_basic_engine_calls(&client).await
}
