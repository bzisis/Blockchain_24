use http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

/// Error thrown when parsing CORS domains went wrong.
///
/// This enumeration represents possible errors that can occur when parsing the CORS domains.
#[derive(Debug, thiserror::Error)]
pub enum CorsDomainError {
    /// Error indicating that the provided domain is an invalid header value.
    #[error("{domain} is an invalid header value")]
    InvalidHeader {
        /// The invalid domain.
        domain: String,
    },
    /// Error indicating that a wildcard origin (`*`) cannot be part of a list of domains.
    #[error("wildcard origin (`*`) cannot be passed as part of a list: {input}")]
    WildCardNotAllowed {
        /// The input string that contains the wildcard.
        input: String,
    },
}

/// Creates a [`CorsLayer`] from the given domains.
///
/// This function creates a CORS layer configuration based on the provided domains. It supports
/// allowing all origins or a list of specific origins.
///
/// # Arguments
///
/// * `http_cors_domains` - A string slice containing the CORS domains separated by commas.
///
/// # Returns
///
/// * `Result<CorsLayer, CorsDomainError>` - A result containing the configured `CorsLayer` or an error.
///
/// # Errors
///
/// This function returns a `CorsDomainError` if the provided domains are invalid or if a wildcard
/// origin (`*`) is included in a list of domains.
///
/// # Examples
///
/// ```
/// let cors_layer = create_cors_layer("*").unwrap();
/// let cors_layer = create_cors_layer("http://example.com,http://example.org").unwrap();
/// ```
pub(crate) fn create_cors_layer(http_cors_domains: &str) -> Result<CorsLayer, CorsDomainError> {
    let cors = match http_cors_domains.trim() {
        "*" => CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers(Any),
        _ => {
            let iter = http_cors_domains.split(',');
            if iter.clone().any(|o| o == "*") {
                return Err(CorsDomainError::WildCardNotAllowed {
                    input: http_cors_domains.to_string(),
                })
            }

            let origins = iter
                .map(|domain| {
                    domain
                        .parse::<HeaderValue>()
                        .map_err(|_| CorsDomainError::InvalidHeader { domain: domain.to_string() })
                })
                .collect::<Result<Vec<HeaderValue>, _>>()?;

            let origin = AllowOrigin::list(origins);
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(origin)
                .allow_headers(Any)
        }
    };
    Ok(cors)
}
