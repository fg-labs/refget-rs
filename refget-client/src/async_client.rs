//! Async HTTP client for the refget API.

use refget_model::{ComparisonResult, SeqCol, SeqColLevel1, SequenceMetadata, SequenceServiceInfo};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde_json::Value;

use crate::error::{ClientError, ClientResult};
use crate::response::{self, MetadataResponse};

/// Async HTTP client for a refget server.
///
/// Supports the GA4GH refget Sequences v2.0.0 and Sequence Collections v1.0.0 APIs.
pub struct RefgetClient {
    client: Client,
    base_url: String,
}

impl RefgetClient {
    /// Create a new client with a default `reqwest::Client`.
    pub fn new(base_url: &str) -> ClientResult<Self> {
        let client = Client::builder().build().map_err(ClientError::Http)?;
        Self::with_client(client, base_url)
    }

    /// Create a new client with a pre-configured `reqwest::Client`.
    pub fn with_client(client: Client, base_url: &str) -> ClientResult<Self> {
        if base_url.is_empty() {
            return Err(ClientError::InvalidUrl("base URL must not be empty".to_string()));
        }
        Ok(Self { client, base_url: base_url.trim_end_matches('/').to_string() })
    }

    /// Send a request and handle the response status.
    ///
    /// Returns `Ok(Some(response))` for 2xx, `Ok(None)` for 404,
    /// and `Err(ServerError)` for other non-2xx statuses.
    async fn send_optional(&self, req: RequestBuilder) -> ClientResult<Option<Response>> {
        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            Ok(Some(resp))
        } else if status == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(ClientError::ServerError { status: status.as_u16(), body })
        }
    }

    /// Send a request that must succeed (404 is an error, not `None`).
    async fn send_required(&self, req: RequestBuilder) -> ClientResult<Response> {
        let resp = req.send().await?;
        if resp.status().is_success() {
            Ok(resp)
        } else {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(ClientError::ServerError { status, body })
        }
    }

    // --- Sequences API ---

    /// Retrieve a sequence (or subsequence) by digest.
    ///
    /// With `start` and/or `end` set, retrieves a subsequence (0-based, half-open).
    /// The server handles defaulting omitted bounds.
    pub async fn get_sequence(
        &self,
        digest: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> ClientResult<Option<Vec<u8>>> {
        let mut req = self.client.get(format!("{}/sequence/{digest}", self.base_url));
        if let Some(s) = start {
            req = req.query(&[("start", s)]);
        }
        if let Some(e) = end {
            req = req.query(&[("end", e)]);
        }
        match self.send_optional(req).await? {
            Some(resp) => Ok(Some(resp.bytes().await?.to_vec())),
            None => Ok(None),
        }
    }

    /// Retrieve metadata for a sequence by digest.
    pub async fn get_metadata(&self, digest: &str) -> ClientResult<Option<SequenceMetadata>> {
        let req = self.client.get(format!("{}/sequence/{digest}/metadata", self.base_url));
        match self.send_optional(req).await? {
            Some(resp) => {
                let envelope: MetadataResponse = resp.json().await?;
                Ok(Some(envelope.metadata))
            }
            None => Ok(None),
        }
    }

    /// Retrieve the sequence service-info.
    pub async fn get_sequence_service_info(&self) -> ClientResult<SequenceServiceInfo> {
        let req = self.client.get(format!("{}/sequence/service-info", self.base_url));
        let resp = self.send_required(req).await?;
        let value: Value = resp.json().await?;
        response::deserialize_sequence_service_info(value).map_err(ClientError::Deserialize)
    }

    // --- Sequence Collections API ---

    /// Retrieve a collection at Level 0 (single digest string).
    pub async fn get_collection_level0(&self, digest: &str) -> ClientResult<Option<String>> {
        let req = self
            .client
            .get(format!("{}/collection/{digest}", self.base_url))
            .query(&[("level", "0")]);
        match self.send_optional(req).await? {
            Some(resp) => {
                let value: Value = resp.json().await?;
                match value.as_str() {
                    Some(s) => Ok(Some(s.to_string())),
                    None => Ok(Some(value.to_string())),
                }
            }
            None => Ok(None),
        }
    }

    /// Retrieve a collection at Level 1 (per-attribute digests).
    pub async fn get_collection_level1(&self, digest: &str) -> ClientResult<Option<SeqColLevel1>> {
        let req = self
            .client
            .get(format!("{}/collection/{digest}", self.base_url))
            .query(&[("level", "1")]);
        match self.send_optional(req).await? {
            Some(resp) => Ok(Some(resp.json().await?)),
            None => Ok(None),
        }
    }

    /// Retrieve a collection at Level 2 (full arrays).
    pub async fn get_collection_level2(&self, digest: &str) -> ClientResult<Option<SeqCol>> {
        let req = self
            .client
            .get(format!("{}/collection/{digest}", self.base_url))
            .query(&[("level", "2")]);
        match self.send_optional(req).await? {
            Some(resp) => Ok(Some(resp.json().await?)),
            None => Ok(None),
        }
    }

    /// Retrieve a collection at an arbitrary level as raw JSON.
    pub async fn get_collection_raw(&self, digest: &str, level: u8) -> ClientResult<Option<Value>> {
        let req = self
            .client
            .get(format!("{}/collection/{digest}", self.base_url))
            .query(&[("level", level.to_string())]);
        match self.send_optional(req).await? {
            Some(resp) => Ok(Some(resp.json().await?)),
            None => Ok(None),
        }
    }

    /// Compare two collections by their digests.
    pub async fn compare_collections(
        &self,
        digest_a: &str,
        digest_b: &str,
    ) -> ClientResult<ComparisonResult> {
        let req = self.client.get(format!("{}/comparison/{digest_a}/{digest_b}", self.base_url));
        Ok(self.send_required(req).await?.json().await?)
    }

    /// Compare a stored collection (by digest) with a provided collection via POST.
    pub async fn compare_collection_with(
        &self,
        digest: &str,
        collection: &SeqCol,
    ) -> ClientResult<ComparisonResult> {
        let req =
            self.client.post(format!("{}/comparison/{digest}", self.base_url)).json(collection);
        Ok(self.send_required(req).await?.json().await?)
    }

    /// List collections with optional filters and pagination.
    ///
    /// Returns the raw JSON response which includes `items`, `total`, `page`, and `page_size`.
    pub async fn list_collections(
        &self,
        filters: &[(&str, &str)],
        page: usize,
        page_size: usize,
    ) -> ClientResult<Value> {
        let mut req = self
            .client
            .get(format!("{}/list/collection", self.base_url))
            .query(&[("page", page.to_string()), ("page_size", page_size.to_string())]);
        for (key, value) in filters {
            req = req.query(&[(key, value)]);
        }
        Ok(self.send_required(req).await?.json().await?)
    }

    /// Get a single attribute array by attribute name and its digest.
    pub async fn get_attribute(&self, attr: &str, digest: &str) -> ClientResult<Option<Value>> {
        let req =
            self.client.get(format!("{}/attribute/collection/{attr}/{digest}", self.base_url));
        match self.send_optional(req).await? {
            Some(resp) => Ok(Some(resp.json().await?)),
            None => Ok(None),
        }
    }

    /// Get the sequence collections service-info as raw JSON.
    pub async fn get_seqcol_service_info(&self) -> ClientResult<Value> {
        let req = self.client.get(format!("{}/service-info", self.base_url));
        Ok(self.send_required(req).await?.json().await?)
    }
}
