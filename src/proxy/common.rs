use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality_pbk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality_sid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub transport_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

pub fn extract_name(uri: &str) -> (&str, String) {
    match uri.rfind('#') {
        Some(pos) => (
            &uri[..pos],
            urlencoding::decode(&uri[pos + 1..])
                .unwrap_or_default()
                .into_owned(),
        ),
        None => (uri, String::new()),
    }
}

pub fn decode_base64(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if let Ok(result) = base64::engine::general_purpose::STANDARD.decode(s) {
        return Ok(result);
    }
    base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(s)
        .map_err(|e| e.into())
}

impl TlsConfig {
    pub fn from_params(params: &HashMap<String, String>, default_enabled: bool) -> Option<Self> {
        let security = params.get("security").map(|s| s.as_str());
        let tls_enabled =
            security == Some("tls") || security == Some("reality") || default_enabled;

        if !tls_enabled {
            return None;
        }

        Some(Self {
            enabled: true,
            server_name: params.get("sni").cloned().filter(|s| !s.is_empty()),
            alpn: params
                .get("alpn")
                .map(|s| s.split(',').map(|a| a.trim().to_string()).collect())
                .filter(|v: &Vec<String>| !v.is_empty()),
            fingerprint: params.get("fp").cloned().filter(|s| !s.is_empty()),
            security: security.map(|s| s.to_string()),
            reality_pbk: params.get("pbk").cloned().filter(|s| !s.is_empty()),
            reality_sid: params.get("sid").cloned().filter(|s| !s.is_empty()),
        })
    }
}

impl TransportConfig {
    pub fn from_params(params: &HashMap<String, String>) -> Option<Self> {
        let transport_type = params
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string());

        if transport_type == "tcp" {
            return None;
        }

        Some(Self {
            transport_type,
            path: params.get("path").cloned().filter(|s| !s.is_empty()),
            host: params.get("host").cloned().filter(|s| !s.is_empty()),
            service_name: params.get("serviceName").cloned().filter(|s| !s.is_empty()),
            headers: None,
        })
    }
}
