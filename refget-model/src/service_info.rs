//! GA4GH service-info types.

use serde::{Deserialize, Serialize};

/// GA4GH service type descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceType {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

/// GA4GH service-info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<serde_json::Value>,
}

/// Extended service-info for the refget Sequences API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceServiceInfo {
    #[serde(flatten)]
    pub service: ServiceInfo,
    pub refget: RefgetServiceDetails,
}

/// Details specific to refget service-info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefgetServiceDetails {
    /// Whether circular sequence retrieval is supported.
    pub circular_supported: bool,
    /// Supported hash algorithms (e.g. `["md5", "ga4gh"]`).
    pub algorithms: Vec<String>,
    /// Supported identifier types (e.g. `["ga4gh", "md5"]`).
    pub identifier_types: Vec<String>,
    /// Maximum length of a subsequence request. 0 means no limit.
    #[serde(default)]
    pub subsequence_limit: u64,
    /// API versions supported by this server (e.g. `["2.0.0"]`).
    #[serde(default)]
    pub supported_api_versions: Vec<String>,
}
