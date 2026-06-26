use std::process::Command;

const PROXY_ADDR: &str = "127.0.0.1:10808";

#[cfg(target_os = "windows")]
const REG_PATH: &str = "HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

pub fn set_system_proxy(enable: bool) -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        return Err("系统代理功能仅支持 Windows".to_string());
    }

    #[cfg(target_os = "windows")]
    {
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
                        REG_PATH, PROXY_ADDR
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
}

pub fn get_system_proxy_status() -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
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
}
