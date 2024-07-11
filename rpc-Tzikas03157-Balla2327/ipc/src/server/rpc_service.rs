//! JSON-RPC service middleware.

use futures_util::future::BoxFuture;
use jsonrpsee::{
    server::{
        middleware::rpc::{ResponseFuture, RpcServiceT},
        IdProvider,
    },
    types::{error::reject_too_many_subscriptions, ErrorCode, ErrorObject, Request},
    BoundedSubscriptions, ConnectionId, Extensions, MethodCallback, MethodResponse, MethodSink,
    Methods, SubscriptionState,
};
use std::sync::Arc;

/// JSON-RPC service middleware.
#[derive(Clone, Debug)]
pub struct RpcService {
    /// The connection ID associated with this service.
    conn_id: ConnectionId,
    /// Registered methods for the RPC service.
    methods: Methods,
    /// Maximum allowed size for the response body.
    max_response_body_size: usize,
    /// Configuration for the RPC service.
    cfg: RpcServiceCfg,
}

/// Configuration of the `RpcService`.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum RpcServiceCfg {
    /// The server supports only calls.
    OnlyCalls,
    /// The server supports both method calls and subscriptions.
    CallsAndSubscriptions {
        /// Bounded subscriptions to control the number of active subscriptions.
        bounded_subscriptions: BoundedSubscriptions,
        /// Sink to send method responses.
        sink: MethodSink,
        /// Provider for unique identifiers.
        id_provider: Arc<dyn IdProvider>,
    },
}

impl RpcService {
    /// Create a new RPC service.
    ///
    /// # Arguments
    ///
    /// * `methods` - The registered methods for the RPC service.
    /// * `max_response_body_size` - Maximum allowed size for the response body.
    /// * `conn_id` - The connection ID associated with this service.
    /// * `cfg` - Configuration for the RPC service.
    pub(crate) const fn new(
        methods: Methods,
        max_response_body_size: usize,
        conn_id: ConnectionId,
        cfg: RpcServiceCfg,
    ) -> Self {
        Self { methods, max_response_body_size, conn_id, cfg }
    }
}

impl<'a> RpcServiceT<'a> for RpcService {
    /// The future type representing the response from an RPC call.
    type Future = ResponseFuture<BoxFuture<'a, MethodResponse>>;

    /// Calls the appropriate method based on the request.
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming RPC request.
    ///
    /// # Returns
    ///
    /// A future that resolves to the method response.
    fn call(&self, req: Request<'a>) -> Self::Future {
        let conn_id = self.conn_id;
        let max_response_body_size = self.max_response_body_size;

        let params = req.params();
        let name = req.method_name();
        let id = req.id().clone();
        let extensions = Extensions::new();

        match self.methods.method_with_name(name) {
            None => {
                // Method not found
                let rp = MethodResponse::error(id, ErrorObject::from(ErrorCode::MethodNotFound));
                ResponseFuture::ready(rp)
            }
            Some((_name, method)) => match method {
                MethodCallback::Sync(callback) => {
                    // Synchronous method callback
                    let rp = (callback)(id, params, max_response_body_size, extensions);
                    ResponseFuture::ready(rp)
                }
                MethodCallback::Async(callback) => {
                    // Asynchronous method callback
                    let params = params.into_owned();
                    let id = id.into_owned();

                    let fut = (callback)(id, params, conn_id, max_response_body_size, extensions);
                    ResponseFuture::future(fut)
                }
                MethodCallback::Subscription(callback) => {
                    // Subscription method callback
                    let RpcServiceCfg::CallsAndSubscriptions {
                        bounded_subscriptions,
                        sink,
                        id_provider,
                    } = self.cfg.clone()
                    else {
                        tracing::warn!("Subscriptions not supported");
                        let rp =
                            MethodResponse::error(id, ErrorObject::from(ErrorCode::InternalError));
                        return ResponseFuture::ready(rp);
                    };

                    if let Some(p) = bounded_subscriptions.acquire() {
                        let conn_state = SubscriptionState {
                            conn_id,
                            id_provider: &*id_provider.clone(),
                            subscription_permit: p,
                        };

                        let fut = callback(id.clone(), params, sink, conn_state, extensions);
                        ResponseFuture::future(fut)
                    } else {
                        let max = bounded_subscriptions.max();
                        let rp = MethodResponse::error(id, reject_too_many_subscriptions(max));
                        ResponseFuture::ready(rp)
                    }
                }
                MethodCallback::Unsubscription(callback) => {
                    // Unsubscription method callback

                    let RpcServiceCfg::CallsAndSubscriptions { .. } = self.cfg else {
                        tracing::warn!("Subscriptions not supported");
                        let rp =
                            MethodResponse::error(id, ErrorObject::from(ErrorCode::InternalError));
                        return ResponseFuture::ready(rp);
                    };

                    let rp = callback(id, params, conn_id, max_response_body_size, extensions);
                    ResponseFuture::ready(rp)
                }
            },
        }
    }
}
