use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_url_input(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());

    // 清除背景
    f.render_widget(Clear, area);

    let block = Block::default()
        .title("添加订阅 (Enter 确认, Esc 取消)")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let input_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(inner);

    // 显示输入内容，带光标
    let input_text = if app.url_input.is_empty() {
        "请输入订阅 URL...".to_string()
    } else {
        format!("{}▌", app.url_input)
    };

    let style = if app.url_input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(input_text)
        .style(style)
        .wrap(Wrap { trim: false });

    f.render_widget(input, input_layout[0]);

    // 如果正在加载，显示加载提示
    if app.loading {
        let loading_area = centered_rect(40, 10, f.area());
        f.render_widget(Clear, loading_area);
        let loading_block = Block::default()
            .title("提示")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::Yellow));
        let loading_text = Paragraph::new("正在拉取订阅，请稍候...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(loading_block, loading_area);
        let inner = Block::default().borders(Borders::ALL).inner(loading_area);
        f.render_widget(loading_text, inner);
    }
}

pub fn render_agency_select(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 40, f.area());

    // 清除背景
    f.render_widget(Clear, area);

    let block = Block::default()
        .title("选择代理商 (Enter 确认, Esc 取消)")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 构建代理商列表
    let items: Vec<String> = app
        .agencies
        .iter()
        .enumerate()
        .map(|(i, agency)| {
            let provider = agency
                .info
                .as_ref()
                .and_then(|info| info.provider.as_deref())
                .unwrap_or("未知供应商");
            let node_count = agency.node.len();
            let marker = if i == app.agency_selected { ">> " } else { "   " };
            format!("{}{} ({} 个节点)", marker, provider, node_count)
        })
        .collect();

    let list_text = items.join("\n");

    let style = Style::default().fg(Color::White);

    let list = Paragraph::new(list_text).style(style);

    f.render_widget(list, inner);
}
