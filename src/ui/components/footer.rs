use ratatui::{
    widgets::Paragraph,
    style::{Color, Style},
};

pub fn render(shortcuts: &str) -> Paragraph<'_> {
    Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::White))
}
