//! HTTP client for GA4GH refget Sequences v2.0.0 and Sequence Collections v1.0.0 APIs.

mod async_client;
#[cfg(feature = "blocking")]
mod blocking;
mod error;
mod response;
#[cfg(feature = "store")]
mod store;

pub use async_client::RefgetClient;
#[cfg(feature = "blocking")]
pub use blocking::RefgetClientBlocking;
pub use error::{ClientError, ClientResult};
#[cfg(feature = "store")]
pub use store::RemoteSequenceStore;
