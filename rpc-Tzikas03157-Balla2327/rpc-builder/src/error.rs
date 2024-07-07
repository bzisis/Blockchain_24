use crate::{cors::CorsDomainError, RethRpcModule};
use reth_ipc::server::IpcServerStartError;
use std::{
    collections::HashSet,
    io::{self, ErrorKind},
    net::SocketAddr,
};

/// Enum representing the different kinds of RPC servers.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ServerKind {
    /// HTTP server.
    Http(SocketAddr),
    /// WebSocket server.
    WS(SocketAddr),
    /// WebSocket and HTTP server on the same port.
    WsHttp(SocketAddr),
    /// Auth server.
    Auth(SocketAddr),
}

impl ServerKind {
    /// Returns the appropriate flags for each variant.
    pub const fn flags(&self) -> &'static str {
        match self {
            Self::Http(_) => "--http.port",
            Self::WS(_) => "--ws.port",
            Self::WsHttp(_) => "--ws.port and --http.port",
            Self::Auth(_) => "--authrpc.port",
        }
    }
}

impl std::fmt::Display for ServerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(addr) => write!(f, "{addr} (HTTP-RPC server)"),
            Self::WS(addr) => write!(f, "{addr} (WS-RPC server)"),
            Self::WsHttp(addr) => write!(f, "{addr} (WS-HTTP-RPC server)"),
            Self::Auth(addr) => write!(f, "{addr} (AUTH server)"),
        }
    }
}

/// Enum representing errors related to the RPC server.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Error thrown during server start.
    #[error("Failed to start {kind} server: {error}")]
    ServerError {
        /// Kind of server.
        kind: ServerKind,
        /// IO error.
        error: io::Error,
    },
    /// Address already in use.
    #[error("address {kind} is already in use (os error 98). Choose a different port using {}", kind.flags())]
    AddressAlreadyInUse {
        /// Kind of server.
        kind: ServerKind,
        /// IO error.
        error: io::Error,
    },
    /// CORS parsing error.
    #[error(transparent)]
    Cors(#[from] CorsDomainError),
    /// HTTP and WebSocket server configured on the same port but with conflicting settings.
    #[error(transparent)]
    WsHttpSamePortError(#[from] WsHttpSamePortError),
    /// Error thrown when IPC server fails to start.
    #[error(transparent)]
    IpcServerError(#[from] IpcServerStartError),
    /// Custom error.
    #[error("{0}")]
    Custom(String),
}

impl RpcError {
    /// Converts an [`io::Error`] to a more descriptive `RpcError`.
    ///
    /// # Arguments
    ///
    /// * `io_error` - The IO error that occurred.
    /// * `kind` - The kind of server that caused the error.
    ///
    /// # Returns
    ///
    /// * `RpcError` - The converted RPC error.
    pub fn server_error(io_error: io::Error, kind: ServerKind) -> Self {
        if io_error.kind() == ErrorKind::AddrInUse {
            return Self::AddressAlreadyInUse { kind, error: io_error }
        }
        Self::ServerError { kind, error: io_error }
    }
}

/// Struct representing conflicting modules between HTTP and WebSocket servers.
#[derive(Debug)]
pub struct ConflictingModules {
    /// Modules present in both HTTP and WebSocket.
    pub overlap: HashSet<RethRpcModule>,
    /// Modules present in HTTP but not in WebSocket.
    pub http_not_ws: HashSet<RethRpcModule>,
    /// Modules present in WebSocket but not in HTTP.
    pub ws_not_http: HashSet<RethRpcModule>,
}

impl std::fmt::Display for ConflictingModules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "different API modules for HTTP and WS on the same port is currently not supported: \
            Overlap: {:?}, \
            HTTP modules not present in WS: {:?} \
            WS modules not present in HTTP: {:?}",
            self.overlap, self.http_not_ws, self.ws_not_http
        )
    }
}

/// Enum representing errors when trying to launch WebSocket and HTTP server on the same port.
#[derive(Debug, thiserror::Error)]
pub enum WsHttpSamePortError {
    /// WebSocket and HTTP server configured on same port but with different CORS domains.
    #[error(
        "CORS domains for HTTP and WS are different, but they are on the same port: \
         HTTP: {http_cors_domains:?}, WS: {ws_cors_domains:?}"
    )]
    ConflictingCorsDomains {
        /// HTTP CORS domains.
        http_cors_domains: Option<String>,
        /// WebSocket CORS domains.
        ws_cors_domains: Option<String>,
    },
    /// WebSocket and HTTP server configured on same port but with different modules.
    #[error("{0}")]
    ConflictingModules(Box<ConflictingModules>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};
    
    /// Tests that the error message for address in use contains the appropriate flags.
    #[test]
    fn test_address_in_use_message() {
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 1234));
        let kinds = [
            ServerKind::Http(addr),
            ServerKind::WS(addr),
            ServerKind::WsHttp(addr),
            ServerKind::Auth(addr),
        ];

        for kind in &kinds {
            let err = RpcError::AddressAlreadyInUse {
                kind: *kind,
                error: io::Error::from(ErrorKind::AddrInUse),
            };

            assert!(err.to_string().contains(kind.flags()));
        }
    }
}
