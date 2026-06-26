use crate::proxy::{self, ProxyNode};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fs;
use std::path::Path;

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
    pub node: Vec<ProxyNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<SubscriptionInfo>,
}

impl Agency {
    pub fn new(url: String, node: Vec<ProxyNode>, info: Option<SubscriptionInfo>) -> Self {
        Self { url, node, info }
    }

    pub fn save_to_config(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config_dir = Path::new("./config");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let filename = format!(
            "config/{}.json",
            self.url
                .replace("https://", "")
                .replace("http://", "")
                .replace("/", "_")
                .replace(":", "_")
        );
        let json = to_string_pretty(self)?;
        fs::write(&filename, json)?;
        Ok(filename)
    }
}

fn parse_traffic_info(header_value: &str) -> Option<TrafficInfo> {
    let mut upload = 0u64;
    let mut download = 0u64;
    let mut total = 0u64;
    let mut expire = None;

    for part in header_value.split(';') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            match key.trim() {
                "upload" => upload = value.trim().parse().unwrap_or(0),
                "download" => download = value.trim().parse().unwrap_or(0),
                "total" => total = value.trim().parse().unwrap_or(0),
                "expire" => expire = value.trim().parse().ok(),
                _ => {}
            }
        }
    }

    Some(TrafficInfo {
        upload,
        download,
        total,
        expire,
    })
}

fn extract_provider_from_url(url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return Some(host.to_string());
        }
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

    // 解析响应头中的供应商和流量信息
    let mut provider = None;
    let mut traffic = None;

    // 尝试从响应头获取供应商
    if let Some(provider_header) = response.headers().get("subscription-userinfo") {
        if let Ok(provider_str) = provider_header.to_str() {
            traffic = parse_traffic_info(provider_str);
        }
    }

    // 尝试从响应头获取供应商名称
    if let Some(provider_name) = response.headers().get("x-provider") {
        if let Ok(name) = provider_name.to_str() {
            provider = Some(name.to_string());
        }
    }

    // 如果没有从响应头获取到供应商，则从URL提取
    if provider.is_none() {
        provider = extract_provider_from_url(sub_url);
    }

    let info = SubscriptionInfo { provider, traffic };

    let data = response.text()?;
    let trimmed = data.trim();

    let decoded: String = match base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        trimmed,
    ) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_subscription() {
        match fetch_subscription(
            "https://103.14.76.98/sub/pianyi/dad9c33c10f77b1d892911351b527e7d",
        ) {
            Ok(agency) => {
                agency.save_to_config().ok();
                println!("{:?}", agency)
            }
            Err(e) => eprintln!("错误: {}", e),
        }
    }

    #[test]
    fn test_parse_traffic_info() {
        let header = "upload=1073741824; download=2147483648; total=10737418240; expire=1735689600";
        let traffic = parse_traffic_info(header).unwrap();

        assert_eq!(traffic.upload, 1073741824);
        assert_eq!(traffic.download, 2147483648);
        assert_eq!(traffic.total, 10737418240);
        assert_eq!(traffic.expire, Some(1735689600));
    }

    #[test]
    fn test_parse_traffic_info_partial() {
        let header = "upload=100; download=200";
        let traffic = parse_traffic_info(header).unwrap();

        assert_eq!(traffic.upload, 100);
        assert_eq!(traffic.download, 200);
        assert_eq!(traffic.total, 0);
        assert_eq!(traffic.expire, None);
    }

    #[test]
    fn test_extract_provider_from_url() {
        let url = "https://example.com/sub/abc123";
        let provider = extract_provider_from_url(url);
        assert_eq!(provider, Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_provider_from_url_invalid() {
        let url = "not a url";
        let provider = extract_provider_from_url(url);
        assert_eq!(provider, None);
    }
}
