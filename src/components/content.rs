use ratatui::{
    widgets::{Block, Borders, List, ListItem},
    style::{Color, Style},
};

pub fn render(items: &[String]) -> List<'_> {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(i.as_str()))
        .collect();

    List::new(list_items)
        .block(Block::default().title("列表").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightBlue))
        .highlight_symbol(">> ")
}