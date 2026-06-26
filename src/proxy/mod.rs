mod common;
mod hy2;
mod ss;
mod trojan;
mod tuic;
mod vless;
mod vmess;

pub use common::{TlsConfig, TransportConfig};
pub use hy2::Hy2Node;
pub use ss::SsNode;
pub use trojan::TrojanNode;
pub use tuic::TuicNode;
pub use vless::VlessNode;
pub use vmess::VmessNode;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProxyNode {
    Vmess(VmessNode),
    Vless(VlessNode),
    Trojan(TrojanNode),
    #[serde(rename = "shadowsocks")]
    Ss(SsNode),
    Tuic(TuicNode),
    #[serde(rename = "hysteria2")]
    Hy2(Hy2Node),
}

impl ProxyNode {
    pub fn protocol_str(&self) -> &str {
        match self {
            Self::Vmess(_) => "vmess",
            Self::Vless(_) => "vless",
            Self::Trojan(_) => "trojan",
            Self::Ss(_) => "ss",
            Self::Tuic(_) => "tuic",
            Self::Hy2(_) => "hy2",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Vmess(n) => &n.name,
            Self::Vless(n) => &n.name,
            Self::Trojan(n) => &n.name,
            Self::Ss(n) => &n.name,
            Self::Tuic(n) => &n.name,
            Self::Hy2(n) => &n.name,
        }
    }

    pub fn address(&self) -> &str {
        match self {
            Self::Vmess(n) => &n.address,
            Self::Vless(n) => &n.address,
            Self::Trojan(n) => &n.address,
            Self::Ss(n) => &n.address,
            Self::Tuic(n) => &n.address,
            Self::Hy2(n) => &n.address,
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            Self::Vmess(n) => n.port,
            Self::Vless(n) => n.port,
            Self::Trojan(n) => n.port,
            Self::Ss(n) => n.port,
            Self::Tuic(n) => n.port,
            Self::Hy2(n) => n.port,
        }
    }
}

pub fn parse_proxy_uri(uri: &str) -> Result<ProxyNode, Box<dyn std::error::Error>> {
    if uri.starts_with("vmess://") {
        return vmess::parse_vmess(uri).map(ProxyNode::Vmess);
    }
    if uri.starts_with("vless://") {
        return vless::parse_vless(uri).map(ProxyNode::Vless);
    }
    if uri.starts_with("trojan://") {
        return trojan::parse_trojan(uri).map(ProxyNode::Trojan);
    }
    if uri.starts_with("ss://") {
        return ss::parse_ss(uri).map(ProxyNode::Ss);
    }
    if uri.starts_with("tuic://") {
        return tuic::parse_tuic(uri).map(ProxyNode::Tuic);
    }
    if uri.starts_with("hysteria2://") || uri.starts_with("hy2://") {
        return hy2::parse_hy2(uri).map(ProxyNode::Hy2);
    }
    Err(format!("Unsupported protocol: {}", uri).into())
}
