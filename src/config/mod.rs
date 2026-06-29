use crate::proxy::{self, ProxyNode};
use dirs::config_dir;
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize)]
pub struct Agency {
    pub url: String,
    pub provider: String,
    pub nodes: Vec<ProxyNode>,
}
impl Agency {
    pub fn new(url: String, provider: String, nodes: Vec<ProxyNode>) -> Self {
        Self {
            url,
            provider,
            nodes,
        }
    }
    pub fn save(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config_dir = config_dir()
            .ok_or("无法获取配置目录")?
            .join("ladderust");
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

pub fn read_config() -> Vec<Agency> {
    let config_dir = match config_dir() {
        Some(dir) => dir.join("ladderust"),
        None => return vec![],
    };
    if !config_dir.exists() {
        if let Err(_) = fs::create_dir_all(&config_dir) {
            return vec![];
        }
    }
    let mut agencies: Vec<Agency> = vec![];
    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json")
                && let Ok(content) = fs::read_to_string(&path)
                && let Ok(agency) = serde_json::from_str::<Agency>(&content)
            {
                agencies.push(agency);
            }
        }
    }
    agencies
}

pub fn extract_provider_from_url(url: &str) -> Option<String> {
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

    Ok(Agency::new(
        sub_url.to_string(),
        extract_provider_from_url(sub_url).unwrap_or_else(|| "未知".to_string()),
        nodes,
    ))
}
