use std::collections::HashMap;
use url::Url;

use super::common::{extract_name, TlsConfig, TransportConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VlessNode {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

pub fn parse_vless(uri: &str) -> Result<VlessNode, Box<dyn std::error::Error>> {
    let (uri_part, name) = extract_name(uri);
    let normalized = uri_part.replace("vless://", "http://");
    let url = Url::parse(&normalized)?;

    let uuid = url.username().to_string();
    let address = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    Ok(VlessNode {
        address,
        port,
        name,
        uuid,
        flow: params.get("flow").cloned().filter(|s| !s.is_empty()),
        encryption: params.get("encryption").cloned().filter(|s| !s.is_empty()),
        transport: TransportConfig::from_params(&params),
        tls: TlsConfig::from_params(&params, false),
    })
}
