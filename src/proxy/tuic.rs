use std::collections::HashMap;
use url::Url;

use super::common::{extract_name, TlsConfig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TuicNode {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub congestion_control: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_relay_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

pub fn parse_tuic(uri: &str) -> Result<TuicNode, Box<dyn std::error::Error>> {
    let (uri_part, name) = extract_name(uri);
    let normalized = uri_part.replace("tuic://", "http://");
    let url = Url::parse(&normalized)?;

    let username = url.username().to_string();
    let password = url
        .password()
        .map(|p| urlencoding::decode(p).unwrap_or_default().into_owned());

    let (uuid, pass) = if let Some((u, p)) = username.split_once(':') {
        (u.to_string(), Some(p.to_string()).or(password))
    } else {
        (username, password)
    };

    let address = url.host_str().unwrap_or("").to_string();
    let port = url.port().unwrap_or(443);
    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    Ok(TuicNode {
        address,
        port,
        name,
        uuid,
        password: pass,
        congestion_control: params.get("congestion_control").cloned(),
        udp_relay_mode: params.get("udp_relay_mode").cloned(),
        tls: Some(TlsConfig {
            enabled: true,
            server_name: params.get("sni").cloned().filter(|s| !s.is_empty()),
            alpn: params
                .get("alpn")
                .map(|s| s.split(',').map(|a| a.trim().to_string()).collect())
                .filter(|v: &Vec<String>| !v.is_empty()),
            fingerprint: None,
            security: None,
            reality_pbk: None,
            reality_sid: None,
        }),
    })
}
