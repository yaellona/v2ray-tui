use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
};
use crate::utils::NodeItem;

pub fn render<'a>(nodes: &[NodeItem]) -> Table<'a> {
    let rows: Vec<Row> = nodes
        .iter()
        .map(|node| {
            Row::new(vec![
                node.protocol.clone(),
                node.name.clone(),
                node.address.clone(),
                node.port.to_string(),
            ])
        })
        .collect();

    let header = Row::new(vec!["协议", "名称", "地址", "端口"])
        .style(Style::default().fg(Color::Yellow))
        .bottom_margin(1);

    Table::new(rows, [
        Constraint::Length(10),
        Constraint::Length(25),
        Constraint::Length(30),
        Constraint::Length(8),
    ])
    .header(header)
    .block(Block::default().title("节点列表").borders(Borders::ALL))
    .row_highlight_style(Style::default().bg(Color::LightBlue))
    .highlight_symbol(">> ")
}
