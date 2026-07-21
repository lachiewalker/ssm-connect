use crate::aws::types::EC2Instance;
use crate::config::Settings;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, instances: &[EC2Instance], selected_instance: Option<usize>, settings: &Settings) {
    let header_cells = ["Name", "Instance ID", "Type", "State", "Private IP", "Forwards"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = instances
        .iter()
        .enumerate()
        .map(|(i, instance)| {
            let style = if Some(i) == selected_instance {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let state_cell = Cell::from(instance.state.as_str())
                .style(Style::default().fg(instance.state.color()));

            let forwards_str = settings
                .enabled_forwards_for(&instance.id)
                .iter()
                .map(|r| r.alias.as_str())
                .collect::<Vec<_>>()
                .join(",");
            let forwards_display = if forwards_str.is_empty() {
                "-".to_string()
            } else {
                forwards_str
            };

            let cells = vec![
                Cell::from(instance.name.clone()),
                Cell::from(instance.id.clone()),
                Cell::from(instance.instance_type.clone()),
                state_cell,
                Cell::from(instance.private_ip.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(forwards_display),
            ];

            Row::new(cells).style(style).height(1)
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Percentage(22),
        ratatui::layout::Constraint::Percentage(22),
        ratatui::layout::Constraint::Percentage(12),
        ratatui::layout::Constraint::Percentage(12),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(17),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("EC2 Instances"));

    f.render_widget(table, area);

    // Show empty state if no instances
    if instances.is_empty() {
        let empty_text = vec![Line::from(vec![
            Span::styled(
                "No instances found in this region",
                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
            ),
        ])];
        let empty = ratatui::widgets::Paragraph::new(empty_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default());

        let centered = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(40),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Percentage(40),
            ])
            .split(area)[1];

        f.render_widget(empty, centered);
    }
}
