//! IPC request handling adapted from [`jsonrpsee`] HTTP request handling.

use futures::{stream::FuturesOrdered, StreamExt};
use jsonrpsee::{
    batch_response_error,
    core::{
        server::helpers::prepare_error,
        tracing::server::{rx_log_from_json, tx_log_from_str},
        JsonRawValue,
    },
    server::middleware::rpc::RpcServiceT,
    types::{
        error::{reject_too_big_request, ErrorCode},
        ErrorObject, Id, InvalidRequest, Notification, Request,
    },
    BatchResponseBuilder, MethodResponse, ResponsePayload,
};
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;
use tokio_util::either::Either;
use tracing::instrument;

/// Alias for a Notification type with an optional JSON raw value.
type Notif<'a> = Notification<'a, Option<&'a JsonRawValue>>;

/// Represents a batch of requests to be processed by the RPC service.
#[derive(Debug, Clone)]
pub(crate) struct Batch<S> {
    data: Vec<u8>,
    rpc_service: S,
}

/// Processes a batch of JSON-RPC requests.
///
/// This function reads the results from each request in the batch, processes them,
/// and sends the complete batch response back to the client.
///
/// # Arguments
///
/// * `b` - A `Batch` struct containing the request data and the RPC service.
/// * `max_response_body_size` - The maximum allowed size for the response body.
///
/// # Returns
///
/// An optional string containing the batch response, or `None` if there were no responses.
#[instrument(name = "batch", skip(b), level = "TRACE")]
pub(crate) async fn process_batch_request<S>(
    b: Batch<S>,
    max_response_body_size: usize,
) -> Option<String>
where
    for<'a> S: RpcServiceT<'a> + Send,
{
    let Batch { data, rpc_service } = b;

    if let Ok(batch) = serde_json::from_slice::<Vec<&JsonRawValue>>(&data) {
        let mut got_notif = false;
        let mut batch_response = BatchResponseBuilder::new_with_limit(max_response_body_size);

        let mut pending_calls: FuturesOrdered<_> = batch
            .into_iter()
            .filter_map(|v| {
                if let Ok(req) = serde_json::from_str::<Request<'_>>(v.get()) {
                    Some(Either::Right(rpc_service.call(req)))
                } else if let Ok(_notif) = serde_json::from_str::<Notif<'_>>(v.get()) {
                    // Notifications should not be answered.
                    got_notif = true;
                    None
                } else {
                    // Valid JSON but not parsable as `InvalidRequest`.
                    let id = match serde_json::from_str::<InvalidRequest<'_>>(v.get()) {
                        Ok(err) => err.id,
                        Err(_) => Id::Null,
                    };

                    Some(Either::Left(async {
                        MethodResponse::error(id, ErrorObject::from(ErrorCode::InvalidRequest))
                    }))
                }
            })
            .collect();

        while let Some(response) = pending_calls.next().await {
            if let Err(too_large) = batch_response.append(&response) {
                return Some(too_large.to_result())
            }
        }

        if got_notif && batch_response.is_empty() {
            None
        } else {
            let batch_resp = batch_response.finish();
            Some(MethodResponse::from_batch(batch_resp).to_result())
        }
    } else {
        Some(batch_response_error(Id::Null, ErrorObject::from(ErrorCode::ParseError)))
    }
}

/// Processes a single JSON-RPC request.
///
/// This function reads the request, processes it, and returns the response.
///
/// # Arguments
///
/// * `data` - A vector of bytes containing the request data.
/// * `rpc_service` - The RPC service to handle the request.
///
/// # Returns
///
/// An optional `MethodResponse` containing the response, or `None` if it was a notification.
pub(crate) async fn process_single_request<S>(
    data: Vec<u8>,
    rpc_service: &S,
) -> Option<MethodResponse>
where
    for<'a> S: RpcServiceT<'a> + Send,
{
    if let Ok(req) = serde_json::from_slice::<Request<'_>>(&data) {
        Some(execute_call_with_tracing(req, rpc_service).await)
    } else if serde_json::from_slice::<Notif<'_>>(&data).is_ok() {
        None
    } else {
        let (id, code) = prepare_error(&data);
        Some(MethodResponse::error(id, ErrorObject::from(code)))
    }
}

/// Executes a JSON-RPC method call with tracing.
///
/// This function logs the method call and delegates it to the RPC service.
///
/// # Arguments
///
/// * `req` - The JSON-RPC request.
/// * `rpc_service` - The RPC service to handle the request.
///
/// # Returns
///
/// A `MethodResponse` containing the response.
#[instrument(name = "method_call", fields(method = req.method.as_ref()), skip(req, rpc_service), level = "TRACE")]
pub(crate) async fn execute_call_with_tracing<'a, S>(
    req: Request<'a>,
    rpc_service: &S,
) -> MethodResponse
where
    for<'b> S: RpcServiceT<'b> + Send,
{
    rpc_service.call(req).await
}

/// Executes a JSON-RPC notification with tracing.
///
/// This function logs the notification.
///
/// # Arguments
///
/// * `notif` - The notification.
/// * `max_log_length` - The maximum length for logging the notification.
#[instrument(name = "notification", fields(method = notif.method.as_ref()), skip(notif, max_log_length), level = "TRACE")]
fn execute_notification(notif: &Notif<'_>, max_log_length: u32) -> MethodResponse {
    rx_log_from_json(notif, max_log_length);
    let response =
        MethodResponse::response(Id::Null, ResponsePayload::success(String::new()), usize::MAX);
    tx_log_from_str(response.as_result(), max_log_length);
    response
}

/// Handles an incoming JSON-RPC request.
///
/// This function determines whether the request is a single request or a batch request,
/// processes it accordingly, and returns the response.
///
/// # Arguments
///
/// * `request` - A string containing the request.
/// * `rpc_service` - The RPC service to handle the request.
/// * `max_response_body_size` - The maximum allowed size for the response body.
/// * `max_request_body_size` - The maximum allowed size for the request body.
/// * `conn` - A semaphore permit to control the number of concurrent requests.
///
/// # Returns
///
/// An optional string containing the response, or `None` if there was no response.
pub(crate) async fn call_with_service<S>(
    request: String,
    rpc_service: S,
    max_response_body_size: usize,
    max_request_body_size: usize,
    conn: Arc<OwnedSemaphorePermit>,
) -> Option<String>
where
    for<'a> S: RpcServiceT<'a> + Send,
{
    enum Kind {
        Single,
        Batch,
    }

    // Determine if the request is a single request or a batch request.
    let request_kind = request
        .chars()
        .find_map(|c| match c {
            '{' => Some(Kind::Single),
            '[' => Some(Kind::Batch),
            _ => None,
        })
        .unwrap_or(Kind::Single);

    let data = request.into_bytes();
    if data.len() > max_request_body_size {
        return Some(batch_response_error(
            Id::Null,
            reject_too_big_request(max_request_body_size as u32),
        ))
    }

    // Process the request based on its kind (single or batch).
    let res = if matches!(request_kind, Kind::Single) {
        let response = process_single_request(data, &rpc_service).await;
        match response {
            Some(response) if response.is_method_call() => Some(response.to_result()),
            _ => {
                // Subscription responses are sent directly over the sink. Returning a response here
                // would lead to duplicate responses for the subscription response.
                None
            }
        }
    } else {
        process_batch_request(Batch { data, rpc_service }, max_response_body_size).await
    };

    // Release the semaphore permit after processing the request.
    drop(conn);

    res
}
