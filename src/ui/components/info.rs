use crate::app::App;
use crate::singbox;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(app: &App) -> Paragraph<'_> {
    let mut info_lines = Vec::new();

    // 显示代理状态
    if app.proxy_running {
        let node_name = app.get_active_node_name().unwrap_or("未知");
        info_lines.push(format!(
            "\u{1f7e2} 代理运行中 (127.0.0.1:{}) - {}",
            singbox::get_listen_port(),
            node_name
        ));
    } else {
        info_lines.push("\u{1f534} 代理已停止".to_string());
    }

    // 显示供应商信息
    if let Some(agency) = app.agencies.first() {
        if let Some(ref info) = agency.info {
            if let Some(ref provider) = info.provider {
                info_lines.push(format!("供应商: {}", provider));
            }

            // 显示流量信息
            if let Some(ref traffic) = info.traffic {
                let used = traffic.upload + traffic.download;
                let total = traffic.total;
                let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
                let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;

                info_lines.push(format!("流量: {:.2} GB / {:.2} GB", used_gb, total_gb));

                // 显示过期时间
                if let Some(expire) = traffic.expire {
                    let expire_date = chrono::DateTime::from_timestamp(expire as i64, 0);
                    if let Some(date) = expire_date {
                        info_lines.push(format!(
                            "过期: {}",
                            date.format("%Y-%m-%d %H:%M:%S")
                        ));
                    }
                }
            }
        }
    }

    if info_lines.len() == 1 {
        info_lines.push("暂无订阅信息".to_string());
    }

    let info_text = info_lines.join(" | ");

    Paragraph::new(info_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().title("状态信息").borders(Borders::ALL))
}
