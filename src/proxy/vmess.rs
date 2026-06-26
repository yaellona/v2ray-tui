use serde::Deserialize;

use super::common::{decode_base64, TlsConfig, TransportConfig};

#[derive(Debug, Clone, Deserialize)]
struct VmessJson {
    ps: Option<String>,
    add: String,
    port: String,
    id: String,
    aid: Option<String>,
    net: Option<String>,
    host: Option<String>,
    path: Option<String>,
    tls: Option<String>,
    sni: Option<String>,
    alpn: Option<String>,
    fp: Option<String>,
    scy: Option<String>,
    pbk: Option<String>,
    sid: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VmessNode {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub uuid: String,
    pub alter_id: u32,
    pub security: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

pub fn parse_vmess(uri: &str) -> Result<VmessNode, Box<dyn std::error::Error>> {
    let raw = &uri[8..]; // strip "vmess://"
    let decoded = decode_base64(raw)?;
    let json_str = String::from_utf8(decoded)?;
    let v: VmessJson = serde_json::from_str(&json_str)?;

    let port: u16 = v.port.parse().unwrap_or(443);
    let alter_id: u32 = v.aid.as_deref().unwrap_or("0").parse().unwrap_or(0);
    let security = v.scy.filter(|s| !s.is_empty()).unwrap_or_else(|| "auto".to_string());
    let net = v.net.filter(|s| !s.is_empty()).unwrap_or_else(|| "tcp".to_string());

    let transport = if net == "tcp" {
        None
    } else {
        Some(TransportConfig {
            transport_type: net,
            path: v.path.filter(|s| !s.is_empty()),
            host: v.host.filter(|s| !s.is_empty()),
            service_name: None,
            headers: None,
        })
    };

    let tls_enabled = v.tls.as_deref() == Some("tls") || v.tls.as_deref() == Some("reality");
    let tls = if tls_enabled {
        Some(TlsConfig {
            enabled: true,
            server_name: v.sni.filter(|s| !s.is_empty()),
            alpn: v
                .alpn
                .map(|s| s.split(',').map(|a| a.trim().to_string()).collect())
                .filter(|v: &Vec<String>| !v.is_empty()),
            fingerprint: v.fp.filter(|s| !s.is_empty()),
            security: v.tls,
            reality_pbk: v.pbk.filter(|s| !s.is_empty()),
            reality_sid: v.sid.filter(|s| !s.is_empty()),
        })
    } else {
        None
    };

    Ok(VmessNode {
        address: v.add,
        port,
        name: v.ps.unwrap_or_default(),
        uuid: v.id,
        alter_id,
        security,
        transport,
        tls,
    })
}
