use ratatui::{
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
};

pub fn render(title: &str) -> Paragraph<'_> {
    Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL))
}