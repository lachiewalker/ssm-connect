use crate::app::get_aws_regions;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_with_data(f: &mut Frame, current_region: &str, selected: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Region list
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Select AWS Region", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" (Current: "),
        Span::styled(current_region, Style::default().fg(Color::Green)),
        Span::raw(")"),
    ]))
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Region list
    let regions = get_aws_regions();
    let items: Vec<ListItem> = regions
        .iter()
        .enumerate()
        .map(|(i, region)| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else if *region == current_region {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let prefix = if i == selected { "> " } else { "  " };
            ListItem::new(format!("{}{}", prefix, region)).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Regions"));
    f.render_widget(list, chunks[1]);

    // Help
    let help_text = vec![Line::from(vec![
        Span::styled("[↑/↓]", Style::default().fg(Color::Green)),
        Span::raw(" Navigate  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Select  "),
        Span::styled("[Esc]", Style::default().fg(Color::Green)),
        Span::raw(" Back"),
    ])];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}
