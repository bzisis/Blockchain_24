//! Helpers for optimism specific RPC implementations.

use std::sync::{atomic::AtomicUsize, Arc};

use jsonrpsee_types::error::{ErrorObject, INTERNAL_ERROR_CODE};
use reqwest::Client;
use reth_rpc_eth_api::RawTransactionForwarder;
use reth_rpc_eth_types::error::{EthApiError, EthResult};
use reth_rpc_types::ToRpcError;

/// Error type when interacting with the Sequencer
#[derive(Debug, thiserror::Error)]
pub enum SequencerRpcError {
    /// Wrapper around an [`reqwest::Error`].
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    /// Thrown when serializing transaction to forward to sequencer
    #[error("invalid sequencer transaction")]
    InvalidSequencerTransaction,
}

impl ToRpcError for SequencerRpcError {
    fn to_rpc_error(&self) -> ErrorObject<'static> {
        ErrorObject::owned(INTERNAL_ERROR_CODE, self.to_string(), None::<String>)
    }
}

impl From<SequencerRpcError> for EthApiError {
    fn from(err: SequencerRpcError) -> Self {
        Self::other(err)
    }
}

/// A client to interact with a Sequencer
#[derive(Debug, Clone)]
pub struct SequencerClient {
    inner: Arc<SequencerClientInner>,
}

impl SequencerClient {
    /// Creates a new [`SequencerClient`] using the default `reqwest` client.
    pub fn new(sequencer_endpoint: impl Into<String>) -> Self {
        // Create a reqwest client using rustls for TLS support.
        let client = Client::builder().use_rustls_tls().build().unwrap();
        Self::with_client(sequencer_endpoint, client)
    }

    /// Creates a new [`SequencerClient`] with a custom HTTP client.
    pub fn with_client(sequencer_endpoint: impl Into<String>, http_client: Client) -> Self {
        // Initialize the inner struct of SequencerClient with provided parameters.
        let inner = SequencerClientInner {
            sequencer_endpoint: sequencer_endpoint.into(),
            http_client,
            id: AtomicUsize::new(0),
        };
        Self { inner: Arc::new(inner) }
    }

    /// Returns the endpoint URL of the sequencer.
    pub fn endpoint(&self) -> &str {
        &self.inner.sequencer_endpoint
    }

    /// Returns a reference to the HTTP client used by the SequencerClient.
    pub fn http_client(&self) -> &Client {
        &self.inner.http_client
    }

    /// Generates and returns the next unique request ID.
    fn next_request_id(&self) -> usize {
        self.inner.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Forwards a raw transaction to the sequencer endpoint asynchronously.
    pub async fn forward_raw_transaction(&self, tx: &[u8]) -> Result<(), SequencerRpcError> {
        // Serialize the transaction into JSON format for RPC request.
        let body = serde_json::to_string(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [format!("0x{}", reth_primitives::hex::encode(tx))],
            "id": self.next_request_id()
        }))
        .map_err(|_| {
            // Log a warning if serialization fails and return InvalidSequencerTransaction error.
            tracing::warn!(
                target = "rpc::eth",
                "Failed to serialize transaction for forwarding to sequencer"
            );
            SequencerRpcError::InvalidSequencerTransaction
        })?;

        // Send the HTTP POST request to the sequencer endpoint with the serialized transaction.
        self.http_client()
            .post(self.endpoint())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(SequencerRpcError::HttpError)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl RawTransactionForwarder for SequencerClient {
    /// Asynchronously forwards a raw transaction to the sequencer endpoint.
    async fn forward_raw_transaction(&self, tx: &[u8]) -> EthResult<()> {
        // Delegate to the non-async function to forward the raw transaction.
        Self::forward_raw_transaction(self, tx).await?;
        Ok(())
    }
}

/// Inner struct for `SequencerClient` holding endpoint URL, HTTP client, and request ID counter.
#[derive(Debug, Default)]
struct SequencerClientInner {
    /// The endpoint URL of the sequencer.
    sequencer_endpoint: String,
    /// The HTTP client used for making requests.
    http_client: Client,
    /// Atomic counter for generating unique request IDs.
    id: AtomicUsize,
}
