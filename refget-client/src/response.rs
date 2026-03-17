//! Response deserialization helpers for refget API responses.

use refget_model::{SequenceMetadata, SequenceServiceInfo};
use serde::Deserialize;

/// Wrapper for the `/sequence/{digest}/metadata` response envelope.
///
/// The server returns `{"metadata": {...}}` — this struct handles unwrapping.
/// The inner object deserializes directly into `SequenceMetadata` since:
/// - `sha512t24u` has `#[serde(rename = "ga4gh")]` so it matches the JSON key
/// - `circular` has `#[serde(default)]` so it defaults to `false` when absent
/// - `trunc512` is an unknown field and is silently ignored by serde
#[derive(Debug, Deserialize)]
pub(crate) struct MetadataResponse {
    pub metadata: SequenceMetadata,
}

/// Deserializes the sequence service-info response.
///
/// The server emits both `"refget"` and `"service"` keys (v1 compat).
/// This tries `"refget"` first, falling back to `"service"`.
pub(crate) fn deserialize_sequence_service_info(
    value: serde_json::Value,
) -> Result<SequenceServiceInfo, serde_json::Error> {
    // Try direct deserialization first (uses the `refget` key via the struct definition)
    match serde_json::from_value::<SequenceServiceInfo>(value.clone()) {
        Ok(info) => Ok(info),
        Err(_) => {
            // Fall back: if there's a "service" key but no "refget", copy it as "refget"
            if let Some(obj) = value.as_object()
                && obj.contains_key("service")
                && !obj.contains_key("refget")
            {
                let mut patched = obj.clone();
                patched.insert("refget".to_string(), obj.get("service").unwrap().clone());
                return serde_json::from_value(serde_json::Value::Object(patched));
            }
            serde_json::from_value(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_metadata_response() {
        let json = serde_json::json!({
            "metadata": {
                "md5": "abc",
                "trunc512": "def",
                "ga4gh": "SQ.ghi",
                "length": 100,
                "aliases": [{"naming_authority": "insdc", "value": "seq1"}]
            }
        });
        let resp: MetadataResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.metadata.md5, "abc");
        assert_eq!(resp.metadata.sha512t24u, "SQ.ghi");
        assert_eq!(resp.metadata.length, 100);
        assert_eq!(resp.metadata.aliases.len(), 1);
        assert!(!resp.metadata.circular);
    }

    #[test]
    fn test_deserialize_metadata_response_without_trunc512() {
        let json = serde_json::json!({
            "metadata": {
                "md5": "abc",
                "ga4gh": "SQ.ghi",
                "length": 100,
                "aliases": []
            }
        });
        let resp: MetadataResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.metadata.md5, "abc");
    }

    #[test]
    fn test_deserialize_service_info_with_refget_key() {
        let json = serde_json::json!({
            "id": "org.ga4gh.refget",
            "name": "test",
            "description": "test server",
            "type": {"group": "org.ga4gh", "artifact": "refget", "version": "2.0.0"},
            "version": "0.1.0",
            "refget": {
                "circular_supported": true,
                "algorithms": ["md5", "ga4gh"],
                "identifier_types": ["ga4gh", "md5"],
                "subsequence_limit": 0,
                "supported_api_versions": ["2.0.0"]
            }
        });
        let info = deserialize_sequence_service_info(json).unwrap();
        assert!(info.refget.circular_supported);
        assert_eq!(info.refget.algorithms.len(), 2);
    }

    #[test]
    fn test_deserialize_service_info_with_service_key_fallback() {
        let json = serde_json::json!({
            "id": "org.ga4gh.refget",
            "name": "test",
            "description": "test server",
            "type": {"group": "org.ga4gh", "artifact": "refget", "version": "2.0.0"},
            "version": "0.1.0",
            "service": {
                "circular_supported": false,
                "algorithms": ["md5"],
                "identifier_types": ["md5"],
                "subsequence_limit": 100,
                "supported_api_versions": ["1.0.0"]
            }
        });
        let info = deserialize_sequence_service_info(json).unwrap();
        assert!(!info.refget.circular_supported);
        assert_eq!(info.refget.subsequence_limit, 100);
    }
}
