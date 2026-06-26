use std::collections::HashMap;
use url::Url;

use super::common::{extract_name, TlsConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Hy2Node {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfs_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfs_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

pub fn parse_hy2(uri: &str) -> Result<Hy2Node, Box<dyn std::error::Error>> {
    let (uri_part, name) = extract_name(uri);
    let normalized = uri_part
        .replacen("hysteria2://", "http://", 1)
        .replacen("hy2://", "http://", 1);
    let url = Url::parse(&normalized)?;

    let password = urlencoding::decode(url.username())?.into_owned();
    let address = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    Ok(Hy2Node {
        address,
        port,
        name,
        password,
        obfs_type: params.get("obfs").cloned().filter(|s| !s.is_empty()),
        obfs_password: params
            .get("obfs-password")
            .cloned()
            .filter(|s| !s.is_empty()),
        tls: Some(TlsConfig {
            enabled: true,
            server_name: params.get("sni").cloned().filter(|s| !s.is_empty()),
            alpn: None,
            fingerprint: None,
            security: None,
            reality_pbk: None,
            reality_sid: None,
        }),
    })
}
