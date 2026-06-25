use reqwest::{Url, header::USER_AGENT};
use base64::Engine;
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Agency {
    pub url: String,
    pub node: Vec<NodeItem>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeItem {
    pub protocol: String,
    pub uuid_or_pass: String,
    pub address: String,
    pub port: u16,
    pub name: String,
    pub params: HashMap<String, String>,
}

impl Agency {
    pub fn new(url: String, node_item: Vec<NodeItem>) -> Self {
        Self { url, node: node_item }
    }

    pub fn save_to_config(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config_dir = Path::new("./config");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let filename = format!("config/{}.json", 
            self.url.replace("https://", "").replace("http://", "").replace("/", "_").replace(":", "_"));
        let json = to_string_pretty(self)?;
        fs::write(&filename, json)?;
        Ok(filename)
    }
}

pub fn parse_proxy_uri(uri: &str) -> Result<NodeItem, Box<dyn std::error::Error>> {
    let (uri_part, name) = match uri.rfind('#') {
        Some(pos) => (&uri[..pos], urlencoding::decode(&uri[pos+1..])?.into_owned()),
        None => (uri, String::new()),
    };

    let normalized = uri_part
        .replace("vless://", "http://")
        .replace("trojan://", "http://")
        .replace("hysteria2://", "http://");

    let url = Url::parse(&normalized)?;

    let protocol = if uri.starts_with("vless://") { "vless" }
        else if uri.starts_with("trojan://") { "trojan" }
        else { "hysteria2" };

    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    Ok(NodeItem {
        protocol: protocol.to_string(),
        uuid_or_pass: url.username().to_string(),
        address: url.host_str().unwrap_or("").to_string(),
        port: url.port().unwrap_or(0),
        name,
        params,
    })
}

pub fn fetch_subscription(sub_url: &str) -> Result<Agency, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(sub_url).header(USER_AGENT, "v2rayN/6.40").send()?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let data = response.text()?;
    let trimmed = data.trim();

    let decoded: String = match base64::engine::general_purpose::STANDARD.decode(trimmed) {
        Ok(bytes) => String::from_utf8(bytes).unwrap_or_default(),
        Err(_) => trimmed.to_string(),
    };

    let nodes: Vec<NodeItem> = decoded
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| parse_proxy_uri(line).ok())
        .collect();

    Ok(Agency::new(sub_url.to_string(), nodes))
}