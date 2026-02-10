use crate::app::SettingsScreenState;
use crate::config::Settings;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &mut SettingsScreenState, settings: &Settings) {
    match state.mode {
        crate::app::SettingsMode::List => render_list(f, state, settings),
        crate::app::SettingsMode::Edit => render_edit(f, state),
    }
}

fn render_list(f: &mut Frame, state: &SettingsScreenState, settings: &Settings) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Command list
            Constraint::Length(3), // Help text
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Auto-Execute Commands")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Description
    let description = Paragraph::new("These commands run automatically on SSM connection")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::NONE));
    let desc_area = Rect {
        x: chunks[0].x,
        y: chunks[0].y + 1,
        width: chunks[0].width,
        height: 1,
    };
    f.render_widget(description, desc_area);

    // Command list
    if settings.auto_execute_commands.is_empty() {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled("No commands configured", Style::default().fg(Color::Gray))),
            Line::from(""),
            Line::from(Span::styled("Press 'a' to add your first command", Style::default().fg(Color::Yellow))),
        ];
        let empty_message = Paragraph::new(empty_lines)
            .block(Block::default().title("Commands").borders(Borders::ALL));
        f.render_widget(empty_message, chunks[1]);
    } else {
        let items: Vec<ListItem> = settings
            .auto_execute_commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let style = if i == state.selected_command {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let content = format!("{}. {}", i + 1, cmd);
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Commands").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        f.render_widget(list, chunks[1]);
    }

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Green)),
            Span::raw(" Navigate  "),
            Span::styled("[a]", Style::default().fg(Color::Green)),
            Span::raw(" Add  "),
            Span::styled("[e]", Style::default().fg(Color::Green)),
            Span::raw(" Edit  "),
            Span::styled("[d]", Style::default().fg(Color::Green)),
            Span::raw(" Delete  "),
            Span::styled("[Esc]", Style::default().fg(Color::Green)),
            Span::raw(" Back to instances"),
        ]),
    ];

    let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn render_edit(f: &mut Frame, state: &mut SettingsScreenState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(2),  // Description
            Constraint::Length(5),  // Text area
            Constraint::Min(1),     // Spacer
            Constraint::Length(3),  // Help text
        ])
        .split(f.area());

    // Title
    let title_text = if state.edit_index.is_some() {
        "Edit Command"
    } else {
        "Add Command"
    };
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Description
    let description = Paragraph::new("Enter a shell command to execute automatically when connecting via SSM")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(description, chunks[1]);

    // Text area
    if let Some(ref textarea) = state.edit_textarea {
        f.render_widget(textarea, chunks[2]);
    }

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Save command  "),
            Span::styled("[Esc]", Style::default().fg(Color::Green)),
            Span::raw(" Cancel and return"),
        ]),
        Line::from(Span::styled("Examples: cd /tmp, sudo su - ubuntu, export PATH=/usr/local/bin:$PATH", Style::default().fg(Color::DarkGray))),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false });
    f.render_widget(help, chunks[4]);
}
