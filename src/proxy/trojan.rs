use std::collections::HashMap;
use url::Url;

use super::common::{extract_name, TlsConfig, TransportConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrojanNode {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

pub fn parse_trojan(uri: &str) -> Result<TrojanNode, Box<dyn std::error::Error>> {
    let (uri_part, name) = extract_name(uri);
    let normalized = uri_part.replace("trojan://", "http://");
    let url = Url::parse(&normalized)?;

    let password = urlencoding::decode(url.username())?.into_owned();
    let address = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    Ok(TrojanNode {
        address,
        port,
        name,
        password,
        transport: TransportConfig::from_params(&params),
        tls: TlsConfig::from_params(&params, true),
    })
}
