use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;
use crate::components;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // 中间
            Constraint::Length(1), // 底部
        ])
        .split(size);

    // 中间表格
    let content = components::content::render(&app.nodes);
    f.render_stateful_widget(content, chunks[0], &mut ratatui::widgets::TableState::default().with_selected(Some(app.selected)));

    // 底部快捷键
    let footer = components::footer::render("q: 退出 | ↑↓: 导航");
    f.render_widget(footer, chunks[1]);
}
