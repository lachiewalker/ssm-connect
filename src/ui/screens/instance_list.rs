use crate::aws::types::EC2Instance;
use crate::config::Settings;
use crate::ui::widgets::instance_table;
use crate::ui::widgets::status_bar;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_with_data(
    f: &mut Frame,
    region: &str,
    instances: &[EC2Instance],
    selected_instance: Option<usize>,
    error_message: &Option<String>,
    info_message: &Option<String>,
    settings: &Settings,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Instance table
            Constraint::Length(3), // Status bar
            Constraint::Length(3), // Error message
        ])
        .split(f.area());

    // Header
    let header_text = vec![Line::from(vec![
        Span::styled("SSM Connect", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | Region: "),
        Span::styled(region, Style::default().fg(Color::Green)),
        Span::raw(format!(" | Instances: {}", instances.len())),
    ])];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Instance table
    instance_table::render(f, chunks[1], instances, selected_instance, settings);

    // Status bar
    status_bar::render(f, chunks[2]);

    // Error or Info message
    if let Some(ref error) = error_message {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error - Press any key to dismiss"));
        f.render_widget(error_text, chunks[3]);
    } else if let Some(ref info) = info_message {
        let info_text = Paragraph::new(info.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Info - Press any key to dismiss"));
        f.render_widget(info_text, chunks[3]);
    }
}
