use std::process::Command;

#[cfg(target_os = "windows")]
const REG_PATH: &str = "HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

pub fn set_system_proxy(enable: bool, port: u16) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let proxy_addr = format!("127.0.0.1:{}", port);
        let value = if enable { "1" } else { "0" };

        let status = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Set-ItemProperty -Path '{}' -Name ProxyEnable -Value {}",
                    REG_PATH, value
                ),
            ])
            .status()
            .map_err(|e| format!("执行命令失败: {}", e))?;

        if !status.success() {
            return Err("设置 ProxyEnable 失败".to_string());
        }

        if enable {
            let status = Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Set-ItemProperty -Path '{}' -Name ProxyServer -Value '{}'",
                        REG_PATH, proxy_addr
                    ),
                ])
                .status()
                .map_err(|e| format!("执行命令失败: {}", e))?;

            if !status.success() {
                return Err("设置 ProxyServer 失败".to_string());
            }
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let proxy_url = format!("http://127.0.0.1:{}", port);

        if enable {
            // 设置当前进程环境变量
            std::env::set_var("http_proxy", &proxy_url);
            std::env::set_var("HTTP_PROXY", &proxy_url);
            std::env::set_var("https_proxy", &proxy_url);
            std::env::set_var("HTTPS_PROXY", &proxy_url);

            // 通过 dbus-update-activation-environment 注入整个 session
            let status = Command::new("dbus-update-activation-environment")
                .args([
                    &format!("http_proxy={}", proxy_url),
                    &format!("HTTP_PROXY={}", proxy_url),
                    &format!("https_proxy={}", proxy_url),
                    &format!("HTTPS_PROXY={}", proxy_url),
                ])
                .status();

            match status {
                Ok(s) if !s.success() => {
                    return Err("dbus-update-activation-environment 执行失败".to_string());
                }
                Err(e) => {
                    return Err(format!("dbus-update-activation-environment 未找到: {}", e));
                }
                _ => {}
            }
        } else {
            // 清空环境变量
            std::env::remove_var("http_proxy");
            std::env::remove_var("HTTP_PROXY");
            std::env::remove_var("https_proxy");
            std::env::remove_var("HTTPS_PROXY");

            let status = Command::new("dbus-update-activation-environment")
                .args(["http_proxy=", "HTTP_PROXY=", "https_proxy=", "HTTPS_PROXY="])
                .status();

            match status {
                Ok(s) if !s.success() => {
                    return Err("dbus-update-activation-environment 执行失败".to_string());
                }
                Err(e) => {
                    return Err(format!("dbus-update-activation-environment 未找到: {}", e));
                }
                _ => {}
            }
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (enable, port);
        Err("系统代理功能仅支持 Windows 和 Linux".to_string())
    }
}

pub fn get_system_proxy_status(port: u16) -> bool {
    #[cfg(target_os = "windows")]
    {
        let _ = port; // Windows 通过注册表 ProxyEnable 检测，不需要端口
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!("(Get-ItemProperty -Path '{}').ProxyEnable", REG_PATH),
            ])
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                stdout == "1"
            }
            Err(_) => false,
        }
    }

    #[cfg(target_os = "linux")]
    {
        let expected = format!("http://127.0.0.1:{}", port);
        std::env::var("http_proxy")
            .or_else(|_| std::env::var("HTTP_PROXY"))
            .or_else(|_| std::env::var("https_proxy"))
            .or_else(|_| std::env::var("HTTPS_PROXY"))
            .map(|v| v == expected)
            .unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = port;
        false
    }
}
