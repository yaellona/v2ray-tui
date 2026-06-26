use crate::app::App;
use crate::singbox;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(app: &App) -> Paragraph<'_> {
    let mut info_lines = Vec::new();

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

    let system_proxy_icon = if app.system_proxy_enabled {
        "\u{1f7e2}"
    } else {
        "\u{1f534}"
    };
    info_lines.push(format!(
        "{} 系统代理: {}",
        system_proxy_icon,
        if app.system_proxy_enabled { "开" } else { "关" }
    ));

    if !app.agencies.is_empty() {
        let current_agency = &app.agencies[app.agency_selected % app.agencies.len()];
        let provider = current_agency
            .info
            .as_ref()
            .and_then(|i| i.provider.as_deref())
            .unwrap_or("未知");
        let node_count = current_agency.nodes.len();
        let mode = if app.viewing_all { "全部" } else { provider };
        info_lines.push(format!(
            "当前代理商: {} ({} 个节点) [{}]",
            provider, node_count, mode
        ));
    }

    if app.loading {
        info_lines.push("正在拉取订阅...".to_string());
    } else if let Some(ref msg) = app.status_message {
        info_lines.push(msg.clone());
    }

    if info_lines.len() == 1 && app.agencies.is_empty() {
        info_lines.push("按 u 添加订阅".to_string());
    }

    let info_text = info_lines.join(" | ");

    Paragraph::new(info_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().title("状态信息").borders(Borders::ALL))
}
