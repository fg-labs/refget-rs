//! Error types for the refget client.

/// Errors that can occur when communicating with a refget server.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Server returned a non-2xx status (other than 404).
    #[error("Server error: HTTP {status}: {body}")]
    ServerError { status: u16, body: String },

    /// Server returned 404.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Failed to deserialize response JSON.
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// Invalid base URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Result type for client operations.
pub type ClientResult<T> = Result<T, ClientError>;
