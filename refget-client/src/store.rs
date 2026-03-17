//! `SequenceStore` bridge for remote refget servers.
//!
//! `RemoteSequenceStore` implements the `SequenceStore` trait, enabling transparent
//! remote-as-local usage of refget sequences.
//!
//! Note: `SeqColStore` is NOT implemented because the trait returns `&SeqCol` (borrowed),
//! which is incompatible with HTTP responses that produce owned values. Use the client
//! methods directly for sequence collection operations.

use refget_model::SequenceMetadata;
use refget_store::{SequenceStore, StoreError, StoreResult};

use crate::RefgetClientBlocking;
use crate::error::ClientError;

/// A `SequenceStore` that fetches sequences from a remote refget server.
///
/// Uses `RefgetClientBlocking` internally for synchronous HTTP calls.
pub struct RemoteSequenceStore {
    client: RefgetClientBlocking,
}

impl RemoteSequenceStore {
    /// Create a new remote store pointing at the given refget server URL.
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        Ok(Self { client: RefgetClientBlocking::new(base_url)? })
    }

    /// Create a new remote store with a pre-configured blocking client.
    pub fn from_client(client: RefgetClientBlocking) -> Self {
        Self { client }
    }
}

/// Map a `ClientError` to a `StoreError`.
fn map_error(e: ClientError) -> StoreError {
    match e {
        ClientError::NotFound(msg) => StoreError::NotFound(msg),
        other => StoreError::Io(std::io::Error::other(other.to_string())),
    }
}

impl SequenceStore for RemoteSequenceStore {
    fn get_sequence(
        &self,
        digest: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> StoreResult<Option<Vec<u8>>> {
        self.client.get_sequence(digest, start, end).map_err(map_error)
    }

    fn get_metadata(&self, digest: &str) -> StoreResult<Option<SequenceMetadata>> {
        self.client.get_metadata(digest).map_err(map_error)
    }

    fn get_length(&self, digest: &str) -> StoreResult<Option<u64>> {
        match self.client.get_metadata(digest).map_err(map_error)? {
            Some(meta) => Ok(Some(meta.length)),
            None => Ok(None),
        }
    }
}
