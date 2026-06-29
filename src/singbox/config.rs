use crate::proxy::{ProxyNode, TlsConfig, TransportConfig};
use serde_json::{Value, json};

pub(crate) fn convert_tls(tls: &Option<TlsConfig>) -> Value {
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

pub(crate) fn convert_transport(transport: &Option<TransportConfig>) -> Option<Value> {
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
            let mut outbound = json!({
                "type": "shadowsocks",
                "tag": "proxy",
                "server": n.address,
                "server_port": n.port,
                "method": n.method,
                "password": n.password,
            });
            if let Some(plugin) = &n.plugin {
                let mut plugin_obj = json!({
                    "enabled": true,
                    "type": plugin,
                });
                if let Some(opts) = &n.plugin_opts {
                    plugin_obj["options"] = json!(opts);
                }
                outbound["plugin"] = plugin_obj;
            }
            outbound
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
                "listen_port": super::LISTEN_PORT
            }
        ],
        "outbounds": [
            node_to_outbound(node),
            { "type": "direct", "tag": "direct" },
            { "type": "block", "tag": "block" }
        ]
    })
}
