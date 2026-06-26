use std::collections::HashMap;
use url::Url;

use super::common::{decode_base64, extract_name};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SsNode {
    pub address: String,
    pub port: u16,
    pub name: String,
    pub method: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_opts: Option<String>,
}

fn parse_ss_plugin(raw: Option<&str>) -> (Option<String>, Option<String>) {
    let raw = match raw {
        Some(s) if !s.is_empty() => s,
        _ => return (None, None),
    };
    let parts: Vec<&str> = raw.split(';').collect();
    let plugin = parts.first().map(|s| s.to_string());
    let opts = if parts.len() > 1 {
        Some(parts[1..].join(";"))
    } else {
        None
    };
    (plugin, opts)
}

pub fn parse_ss(uri: &str) -> Result<SsNode, Box<dyn std::error::Error>> {
    let (uri_part, name) = extract_name(uri);
    let raw = &uri_part[5..]; // strip "ss://"

    if let Some(at_pos) = raw.find('@') {
        let userinfo_b64 = &raw[..at_pos];
        let server_part = &raw[at_pos + 1..];

        let decoded = decode_base64(userinfo_b64)?;
        let userinfo = String::from_utf8(decoded)?;
        let (method, password) = userinfo
            .split_once(':')
            .ok_or("Invalid ss userinfo: missing ':'")?;

        let normalized = format!("http://{}", server_part);
        let url = Url::parse(&normalized)?;
        let address = url.host_str().unwrap_or("").to_string();
        let port = url.port().unwrap_or(8388);

        let params: HashMap<String, String> = url.query_pairs().into_owned().collect();
        let (plugin, plugin_opts) = parse_ss_plugin(params.get("plugin").map(|s| s.as_str()));

        Ok(SsNode {
            address,
            port,
            name,
            method: method.to_string(),
            password: password.to_string(),
            plugin,
            plugin_opts,
        })
    } else {
        // Legacy format: ss://BASE64(method:password@host:port)#name
        let decoded = decode_base64(raw)?;
        let full = String::from_utf8(decoded)?;
        let (userinfo, server) = full
            .split_once('@')
            .ok_or("Invalid ss legacy format: missing '@'")?;
        let (method, password) = userinfo
            .split_once(':')
            .ok_or("Invalid ss userinfo: missing ':'")?;

        // Parse host:port, handle IPv6
        let (host, port_str) = if server.starts_with('[') {
            let end = server.find(']').ok_or("Invalid IPv6 address")?;
            let host = &server[1..end];
            let rest = &server[end + 1..];
            let port = if rest.starts_with(':') {
                &rest[1..]
            } else {
                "8388"
            };
            (host.to_string(), port)
        } else {
            match server.rsplit_once(':') {
                Some((h, p)) => (h.to_string(), p),
                None => (server.to_string(), "8388"),
            }
        };

        let port: u16 = port_str.parse()?;

        Ok(SsNode {
            address: host,
            port,
            name,
            method: method.to_string(),
            password: password.to_string(),
            plugin: None,
            plugin_opts: None,
        })
    }
}
