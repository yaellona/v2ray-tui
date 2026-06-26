mod components;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 信息区域
            Constraint::Min(0),   // 中间表格
            Constraint::Length(1), // 底部
        ])
        .split(size);

    // 信息区域
    let info = components::info::render(app);
    f.render_widget(info, chunks[0]);

    // 中间表格
    let content = components::content::render(&app.nodes);
    f.render_stateful_widget(content, chunks[1], &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)));

    // 底部快捷键
    let footer = components::footer::render("q: 退出 | ↑↓: 导航 | Enter: 启动/停止代理");
    f.render_widget(footer, chunks[2]);
}
