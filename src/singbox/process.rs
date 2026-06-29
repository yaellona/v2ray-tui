use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use crate::proxy::ProxyNode;

fn get_config_path() -> PathBuf {
    std::env::temp_dir().join("v2ray-tui-singbox-config.json")
}

pub fn start_proxy(node: &ProxyNode) -> Result<Child, String> {
    let config = super::config::generate_config(node);
    let config_path = get_config_path();

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, config_json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    let config_path_str = config_path.to_str()
        .ok_or("配置文件路径无效")?;
    let mut child = Command::new("sing-box")
        .args(["run", "-c", config_path_str])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 sing-box 失败: {}", e))?;

    // 短暂等待检查进程是否立即退出（配置错误等）
    std::thread::sleep(std::time::Duration::from_millis(500));
    match child.try_wait() {
        Ok(Some(status)) => {
            let stderr_output = child
                .stderr
                .take()
                .and_then(|mut stderr| {
                    use std::io::Read;
                    let mut buf = String::new();
                    stderr.read_to_string(&mut buf).ok().map(|_| buf)
                })
                .unwrap_or_default();
            let _ = fs::remove_file(&config_path);
            if stderr_output.is_empty() {
                Err(format!("sing-box 启动后立即退出 (状态: {})", status))
            } else {
                Err(format!("sing-box 启动失败: {}", stderr_output.trim()))
            }
        }
        Ok(None) => Ok(child),
        Err(e) => Err(format!("检查 sing-box 进程状态失败: {}", e)),
    }
}

pub fn stop_proxy(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
    let config_path = get_config_path();
    let _ = fs::remove_file(config_path);
}
