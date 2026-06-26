use crate::proxy::{self, ProxyNode};
use dirs::config_dir;
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficInfo {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub provider: Option<String>,
    pub traffic: Option<TrafficInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agency {
    pub url: String,
    pub nodes: Vec<ProxyNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<SubscriptionInfo>,
}

impl Agency {
    pub fn new(url: String, nodes: Vec<ProxyNode>, info: Option<SubscriptionInfo>) -> Self {
        Self { url, nodes, info }
    }

    pub fn save_to_config(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config_dir = config_dir().unwrap().join("ladderust");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let mut hasher = DefaultHasher::new();
        self.url.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        let filename = config_dir.join(format!("{}.json", hash));
        let json = to_string_pretty(self)?;
        fs::write(&filename, json)?;
        Ok(filename.to_string_lossy().to_string())
    }
}

fn parse_traffic_info(header_value: &str) -> Option<TrafficInfo> {
    let mut upload = 0u64;
    let mut download = 0u64;
    let mut total = 0u64;
    let mut expire = None;
    let mut found = false;

    for part in header_value.split(';') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            match key.trim() {
                "upload" => {
                    upload = value.trim().parse().unwrap_or(0);
                    found = true;
                }
                "download" => {
                    download = value.trim().parse().unwrap_or(0);
                    found = true;
                }
                "total" => {
                    total = value.trim().parse().unwrap_or(0);
                    found = true;
                }
                "expire" => {
                    expire = value.trim().parse().ok();
                    found = true;
                }
                _ => {}
            }
        }
    }

    if found {
        Some(TrafficInfo {
            upload,
            download,
            total,
            expire,
        })
    } else {
        None
    }
}

fn extract_provider_from_url(url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(url)
        && let Some(host) = parsed.host_str()
    {
        return Some(host.to_string());
    }
    None
}

pub fn fetch_subscription(sub_url: &str) -> Result<Agency, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(sub_url)
        .header(USER_AGENT, "v2rayN/6.40")
        .send()?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let mut provider = None;
    let mut traffic = None;

    if let Some(provider_header) = response.headers().get("subscription-userinfo")
        && let Ok(provider_str) = provider_header.to_str()
    {
        traffic = parse_traffic_info(provider_str);
    }

    if let Some(provider_name) = response.headers().get("x-provider")
        && let Ok(name) = provider_name.to_str()
    {
        provider = Some(name.to_string());
    }

    if provider.is_none() {
        provider = extract_provider_from_url(sub_url);
    }

    let info = SubscriptionInfo { provider, traffic };

    let data = response.text()?;
    let trimmed = data.trim();

    let decoded: String =
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, trimmed) {
            Ok(bytes) => String::from_utf8(bytes).unwrap_or_default(),
            Err(_) => trimmed.to_string(),
        };

    let nodes: Vec<ProxyNode> = decoded
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| proxy::parse_proxy_uri(line).ok())
        .collect();

    Ok(Agency::new(sub_url.to_string(), nodes, Some(info)))
}
