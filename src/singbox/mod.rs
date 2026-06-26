use crate::proxy::{ProxyNode, TlsConfig, TransportConfig};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

const LISTEN_PORT: u16 = 10808;

fn get_config_path() -> PathBuf {
    std::env::temp_dir().join("v2ray-tui-singbox-config.json")
}

fn convert_tls(tls: &Option<TlsConfig>) -> Value {
    match tls {
        Some(tls) => {
            let mut obj = json!({
                "enabled": tls.enabled,
            });
            if let Some(sni) = &tls.server_name {
                obj["server_name"] = json!(sni);
            }
            if let Some(alpn) = &tls.alpn {
                obj["alpn"] = json!(alpn);
            }
            if let Some(fp) = &tls.fingerprint {
                obj["utls"] = json!({ "enabled": true, "fingerprint": fp });
            }
            if tls.security.as_deref() == Some("reality") {
                let mut reality = json!({ "enabled": true });
                if let Some(pbk) = &tls.reality_pbk {
                    reality["public_key"] = json!(pbk);
                }
                if let Some(sid) = &tls.reality_sid {
                    reality["short_id"] = json!(sid);
                }
                obj["reality"] = reality;
            }
            obj
        }
        None => json!({ "enabled": false }),
    }
}

fn convert_transport(transport: &Option<TransportConfig>) -> Option<Value> {
    transport.as_ref().map(|t| {
        let mut obj = json!({
            "type": t.transport_type,
        });
        if let Some(path) = &t.path {
            obj["path"] = json!(path);
        }
        if let Some(host) = &t.host {
            let mut headers = json!({});
            headers["Host"] = json!(host);
            obj["headers"] = headers;
        }
        if let Some(service_name) = &t.service_name {
            obj["service_name"] = json!(service_name);
        }
        obj
    })
}

fn node_to_outbound(node: &ProxyNode) -> Value {
    match node {
        ProxyNode::Vmess(n) => {
            let mut outbound = json!({
                "type": "vmess",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "uuid": n.uuid,
                "alter_id": n.alter_id,
                "security": n.security,
            });
            outbound["tls"] = convert_tls(&n.tls);
            if let Some(transport) = convert_transport(&n.transport) {
                outbound["transport"] = transport;
            }
            outbound
        }
        ProxyNode::Vless(n) => {
            let mut outbound = json!({
                "type": "vless",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "uuid": n.uuid,
            });
            if let Some(flow) = &n.flow {
                outbound["flow"] = json!(flow);
            }
            outbound["tls"] = convert_tls(&n.tls);
            if let Some(transport) = convert_transport(&n.transport) {
                outbound["transport"] = transport;
            }
            outbound
        }
        ProxyNode::Trojan(n) => {
            let mut outbound = json!({
                "type": "trojan",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "password": n.password,
            });
            outbound["tls"] = convert_tls(&n.tls);
            if let Some(transport) = convert_transport(&n.transport) {
                outbound["transport"] = transport;
            }
            outbound
        }
        ProxyNode::Ss(n) => {
            json!({
                "type": "shadowsocks",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "method": n.method,
                "password": n.password,
            })
        }
        ProxyNode::Tuic(n) => {
            let mut outbound = json!({
                "type": "tuic",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "uuid": n.uuid,
            });
            if let Some(password) = &n.password {
                outbound["password"] = json!(password);
            }
            if let Some(cc) = &n.congestion_control {
                outbound["congestion_control"] = json!(cc);
            }
            if let Some(udp) = &n.udp_relay_mode {
                outbound["udp_relay_mode"] = json!(udp);
            }
            outbound["tls"] = convert_tls(&n.tls);
            outbound
        }
        ProxyNode::Hy2(n) => {
            let mut outbound = json!({
                "type": "hysteria2",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "password": n.password,
            });
            if let Some(obfs_type) = &n.obfs_type {
                let mut obfs = json!({ "type": obfs_type });
                if let Some(password) = &n.obfs_password {
                    obfs["password"] = json!(password);
                }
                outbound["obfs"] = obfs;
            }
            outbound["tls"] = convert_tls(&n.tls);
            outbound
        }
    }
}

pub fn generate_config(node: &ProxyNode) -> Value {
    json!({
        "log": {
            "level": "warn"
        },
        "inbounds": [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": LISTEN_PORT
            }
        ],
        "outbounds": [
            node_to_outbound(node),
            { "type": "direct", "tag": "direct" },
            { "type": "block", "tag": "block" }
        ]
    })
}

pub fn start_proxy(node: &ProxyNode) -> Result<Child, String> {
    let config = generate_config(node);
    let config_path = get_config_path();

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    let child = Command::new("sing-box")
        .args(["run", "-c", config_path.to_str().unwrap()])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 sing-box 失败: {}", e))?;
    Ok(child)
}

pub fn stop_proxy(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
    let config_path = get_config_path();
    let _ = fs::remove_file(config_path);
}

pub fn get_listen_port() -> u16 {
    LISTEN_PORT
}
