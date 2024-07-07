use crate::error::{RpcError, ServerKind};
use http::header::AUTHORIZATION;
use jsonrpsee::{
    core::RegisterMethodError,
    http_client::{transport::HttpBackend, HeaderMap},
    server::{AlreadyStoppedError, RpcModule},
    Methods,
};
use reth_engine_primitives::EngineTypes;
use reth_rpc_api::servers::*;
use reth_rpc_eth_types::EthSubscriptionIdProvider;
use reth_rpc_layer::{
    secret_to_bearer_header, AuthClientLayer, AuthClientService, AuthLayer, JwtAuthValidator,
    JwtSecret,
};
use reth_rpc_server_types::constants;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::layer::util::Identity;

pub use jsonrpsee::server::ServerBuilder;
pub use reth_ipc::server::Builder as IpcServerBuilder;

/// Server configuration for the auth server.
///
/// This structure holds the configuration required to start an authentication-enabled server.
#[derive(Debug)]
pub struct AuthServerConfig {
    /// The socket address where the server should listen.
    pub(crate) socket_addr: SocketAddr,
    /// The secret key used for the JWT authentication layer.
    pub(crate) secret: JwtSecret,
    /// Configuration for the JSON-RPC HTTP server.
    pub(crate) server_config: ServerBuilder<Identity, Identity>,
    /// Optional configuration for the IPC server.
    pub(crate) ipc_server_config: Option<IpcServerBuilder<Identity, Identity>>,
    /// Optional IPC endpoint.
    pub(crate) ipc_endpoint: Option<String>,
}

// === impl AuthServerConfig ===

impl AuthServerConfig {
    /// Convenience function to create a new `AuthServerConfigBuilder` with the given secret.
    ///
    /// # Arguments
    ///
    /// * `secret` - A `JwtSecret` for the auth layer of the server.
    ///
    /// # Returns
    ///
    /// * `AuthServerConfigBuilder` - A builder to configure the `AuthServerConfig`.
    pub const fn builder(secret: JwtSecret) -> AuthServerConfigBuilder {
        AuthServerConfigBuilder::new(secret)
    }

    /// Returns the address the server will listen on.
    ///
    /// # Returns
    ///
    /// * `SocketAddr` - The socket address of the server.
    pub const fn address(&self) -> SocketAddr {
        self.socket_addr
    }

    /// Convenience function to start a server in one step.
    ///
    /// # Arguments
    ///
    /// * `module` - An `AuthRpcModule` containing the RPC methods to be served.
    ///
    /// # Returns
    ///
    /// * `Result<AuthServerHandle, RpcError>` - A result containing the server handle or an error.
    pub async fn start(self, module: AuthRpcModule) -> Result<AuthServerHandle, RpcError> {
        let Self { socket_addr, secret, server_config, ipc_server_config, ipc_endpoint } = self;

        // Create auth middleware.
        let middleware =
            tower::ServiceBuilder::new().layer(AuthLayer::new(JwtAuthValidator::new(secret)));

        // By default, both http and ws are enabled.
        let server = server_config
            .set_http_middleware(middleware)
            .build(socket_addr)
            .await
            .map_err(|err| RpcError::server_error(err, ServerKind::Auth(socket_addr)))?;

        let local_addr = server
            .local_addr()
            .map_err(|err| RpcError::server_error(err, ServerKind::Auth(socket_addr)))?;

        let handle = server.start(module.inner.clone());
        let mut ipc_handle: Option<jsonrpsee::server::ServerHandle> = None;

        if let Some(ipc_server_config) = ipc_server_config {
            let ipc_endpoint_str = ipc_endpoint
                .clone()
                .unwrap_or_else(|| constants::DEFAULT_ENGINE_API_IPC_ENDPOINT.to_string());
            let ipc_server = ipc_server_config.build(ipc_endpoint_str);
            let res = ipc_server
                .start(module.inner)
                .await
                .map_err(reth_ipc::server::IpcServerStartError::from)?;
            ipc_handle = Some(res);
        }

        Ok(AuthServerHandle { handle, local_addr, secret, ipc_endpoint, ipc_handle })
    }
}

/// Builder type for configuring an `AuthServerConfig`.
#[derive(Debug)]
pub struct AuthServerConfigBuilder {
    socket_addr: Option<SocketAddr>,
    secret: JwtSecret,
    server_config: Option<ServerBuilder<Identity, Identity>>,
    ipc_server_config: Option<IpcServerBuilder<Identity, Identity>>,
    ipc_endpoint: Option<String>,
}

// === impl AuthServerConfigBuilder ===

impl AuthServerConfigBuilder {
    /// Create a new `AuthServerConfigBuilder` with the given `secret`.
    ///
    /// # Arguments
    ///
    /// * `secret` - A `JwtSecret` for the auth layer of the server.
    ///
    /// # Returns
    ///
    /// * `AuthServerConfigBuilder` - A new builder instance.
    pub const fn new(secret: JwtSecret) -> Self {
        Self {
            socket_addr: None,
            secret,
            server_config: None,
            ipc_server_config: None,
            ipc_endpoint: None,
        }
    }

    /// Set the socket address for the server.
    ///
    /// # Arguments
    ///
    /// * `socket_addr` - The socket address to bind the server.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance with the socket address set.
    pub const fn socket_addr(mut self, socket_addr: SocketAddr) -> Self {
        self.socket_addr = Some(socket_addr);
        self
    }

    /// Set the socket address for the server if it is present.
    ///
    /// # Arguments
    ///
    /// * `socket_addr` - An optional socket address to bind the server.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance with the socket address set if provided.
    pub const fn maybe_socket_addr(mut self, socket_addr: Option<SocketAddr>) -> Self {
        self.socket_addr = socket_addr;
        self
    }

    /// Set the secret for the server.
    ///
    /// # Arguments
    ///
    /// * `secret` - A `JwtSecret` for the auth layer of the server.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance with the secret set.
    pub const fn secret(mut self, secret: JwtSecret) -> Self {
        self.secret = secret;
        self
    }

    /// Configures the JSON-RPC server.
    ///
    /// Note: This always configures an [`EthSubscriptionIdProvider`].
    ///
    /// # Arguments
    ///
    /// * `config` - A `ServerBuilder` instance with the server configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance with the server configuration set.
    pub fn with_server_config(mut self, config: ServerBuilder<Identity, Identity>) -> Self {
        self.server_config = Some(config.set_id_provider(EthSubscriptionIdProvider::default()));
        self
    }

    /// Set the IPC endpoint for the server.
    ///
    /// # Arguments
    ///
    /// * `ipc_endpoint` - A string representing the IPC endpoint.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance with the IPC endpoint set.
    pub fn ipc_endpoint(mut self, ipc_endpoint: String) -> Self {
        self.ipc_endpoint = Some(ipc_endpoint);
        self
    }

    /// Configures the IPC server.
    ///
    /// Note: This always configures an [`EthSubscriptionIdProvider`].
    ///
    /// # Arguments
    ///
    /// * `config` - An `IpcServerBuilder` instance with the IPC server configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance with the IPC server configuration set.
    pub fn with_ipc_config(mut self, config: IpcServerBuilder<Identity, Identity>) -> Self {
        self.ipc_server_config = Some(config.set_id_provider(EthSubscriptionIdProvider::default()));
        self
    }

    /// Build the `AuthServerConfig`.
    ///
    /// # Returns
    ///
    /// * `AuthServerConfig` - A new `AuthServerConfig` instance with the specified configurations.
    pub fn build(self) -> AuthServerConfig {
        AuthServerConfig {
            socket_addr: self.socket_addr.unwrap_or_else(|| {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), constants::DEFAULT_AUTH_PORT)
            }),
            secret: self.secret,
            server_config: self.server_config.unwrap_or_else(|| {
                ServerBuilder::new()
                    // This needs to be large enough to handle large eth_getLogs responses and maximum
                    // payload bodies limit for `engine_getPayloadBodiesByRangeV`.
                    // ~750MB per response should be enough.
                    .max_response_body_size(750 * 1024 * 1024)
                    // Connections to this server are always authenticated, hence this only affects
                    // connections from the CL or any other client that uses JWT. This should be
                    // more than enough so that the CL (or multiple CL nodes) will never get rate
                    // limited.
                    .max_connections(500)
                    // Bump the default request size slightly. There aren't any methods exposed with
                    // dynamic request params that can exceed this.
                    .max_request_body_size(128 * 1024 * 1024)
                    .set_id_provider(EthSubscriptionIdProvider::default())
            }),
            ipc_server_config: self.ipc_server_config.map(|ipc_server_config| {
                ipc_server_config
                    .max_response_body_size(750 * 1024 * 1024)
                    .max_connections(500)
                    .max_request_body_size(128 * 1024 * 1024)
                    .set_id_provider(EthSubscriptionIdProvider::default())
            }),
            ipc_endpoint: self.ipc_endpoint,
        }
    }
}

/// Holds installed modules for the auth server.
///
/// This structure encapsulates the RPC methods that are served by the auth server.
#[derive(Debug, Clone)]
pub struct AuthRpcModule {
    pub(crate) inner: RpcModule<()>,
}

// === impl AuthRpcModule ===

impl AuthRpcModule {
    /// Create a new `AuthRpcModule` with the given `engine_api`.
    ///
    /// # Arguments
    ///
    /// * `engine` - The engine API to be served.
    ///
    /// # Returns
    ///
    /// * `AuthRpcModule` - A new instance with the provided engine API.
    pub fn new<EngineApi, EngineT>(engine: EngineApi) -> Self
    where
        EngineT: EngineTypes + 'static,
        EngineApi: EngineApiServer<EngineT>,
    {
        let mut module = RpcModule::new(());
        module.merge(engine.into_rpc()).expect("No conflicting methods");
        Self { inner: module }
    }

    /// Get a mutable reference to the inner `RpcModule`.
    ///
    /// # Returns
    ///
    /// * `&mut RpcModule<()>` - A mutable reference to the inner `RpcModule`.
    pub fn module_mut(&mut self) -> &mut RpcModule<()> {
        &mut self.inner
    }

    /// Merge the given [Methods] into the configured authenticated methods.
    ///
    /// Fails if any of the methods in `other` is present already.
    ///
    /// # Arguments
    ///
    /// * `other` - Methods to be merged into the current module.
    ///
    /// # Returns
    ///
    /// * `Result<bool, RegisterMethodError>` - A result indicating success or failure of the merge.
    pub fn merge_auth_methods(
        &mut self,
        other: impl Into<Methods>,
    ) -> Result<bool, RegisterMethodError> {
        self.module_mut().merge(other.into()).map(|_| true)
    }

    /// Convenience function for starting a server.
    ///
    /// # Arguments
    ///
    /// * `config` - An `AuthServerConfig` containing the server configuration.
    ///
    /// # Returns
    ///
    /// * `Result<AuthServerHandle, RpcError>` - A result containing the server handle or an error.
    pub async fn start_server(
        self,
        config: AuthServerConfig,
    ) -> Result<AuthServerHandle, RpcError> {
        config.start(self).await
    }
}

/// A handle to the spawned auth server.
///
/// When this type is dropped or [`AuthServerHandle::stop`] has been called, the server will be
/// stopped.
#[derive(Clone, Debug)]
#[must_use = "Server stops if dropped"]
pub struct AuthServerHandle {
    local_addr: SocketAddr,
    handle: jsonrpsee::server::ServerHandle,
    secret: JwtSecret,
    ipc_endpoint: Option<String>,
    ipc_handle: Option<jsonrpsee::server::ServerHandle>,
}

// === impl AuthServerHandle ===

impl AuthServerHandle {
    /// Returns the [`SocketAddr`] of the HTTP server if started.
    ///
    /// # Returns
    ///
    /// * `SocketAddr` - The socket address of the HTTP server.
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Tell the server to stop without waiting for the server to stop.
    ///
    /// # Returns
    ///
    /// * `Result<(), AlreadyStoppedError>` - A result indicating success or failure of stopping the server.
    pub fn stop(self) -> Result<(), AlreadyStoppedError> {
        self.handle.stop()
    }

    /// Returns the URL to the HTTP server.
    ///
    /// # Returns
    ///
    /// * `String` - The URL of the HTTP server.
    pub fn http_url(&self) -> String {
        format!("http://{}", self.local_addr)
    }

    /// Returns the URL to the WebSocket server.
    ///
    /// # Returns
    ///
    /// * `String` - The URL of the WebSocket server.
    pub fn ws_url(&self) -> String {
        format!("ws://{}", self.local_addr)
    }

    /// Returns an HTTP client connected to the server.
    ///
    /// # Returns
    ///
    /// * `jsonrpsee::http_client::HttpClient<AuthClientService<HttpBackend>>` - The HTTP client.
    pub fn http_client(
        &self,
    ) -> jsonrpsee::http_client::HttpClient<AuthClientService<HttpBackend>> {
        // Create a middleware that adds a new JWT token to every request.
        let secret_layer = AuthClientLayer::new(self.secret);
        let middleware = tower::ServiceBuilder::default().layer(secret_layer);
        jsonrpsee::http_client::HttpClientBuilder::default()
            .set_http_middleware(middleware)
            .build(self.http_url())
            .expect("Failed to create HTTP client")
    }

    /// Returns a WebSocket client connected to the server.
    ///
    /// Note that the connection can only be established within 1 minute due to the JWT token expiration.
    ///
    /// # Returns
    ///
    /// * `jsonrpsee::ws_client::WsClient` - The WebSocket client.
    pub async fn ws_client(&self) -> jsonrpsee::ws_client::WsClient {
        jsonrpsee::ws_client::WsClientBuilder::default()
            .set_headers(HeaderMap::from_iter([(
                AUTHORIZATION,
                secret_to_bearer_header(&self.secret),
            )]))
            .build(self.ws_url())
            .await
            .expect("Failed to create WebSocket client")
    }

    /// Returns an IPC client connected to the server.
    ///
    /// # Returns
    ///
    /// * `Option<jsonrpsee::async_client::Client>` - The IPC client if the endpoint is set.
    #[cfg(unix)]
    pub async fn ipc_client(&self) -> Option<jsonrpsee::async_client::Client> {
        use reth_ipc::client::IpcClientBuilder;

        if let Some(ipc_endpoint) = &self.ipc_endpoint {
            return Some(
                IpcClientBuilder::default()
                    .build(ipc_endpoint)
                    .await
                    .expect("Failed to create IPC client"),
            )
        }
        None
    }

    /// Returns an IPC handle.
    ///
    /// # Returns
    ///
    /// * `Option<jsonrpsee::server::ServerHandle>` - The IPC server handle if available.
    pub fn ipc_handle(&self) -> Option<jsonrpsee::server::ServerHandle> {
        self.ipc_handle.clone()
    }

    /// Returns the IPC endpoint.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The IPC endpoint if set.
    pub fn ipc_endpoint(&self) -> Option<String> {
        self.ipc_endpoint.clone()
    }
}
