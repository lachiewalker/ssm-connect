use crate::app::{PortForwardField, PortForwardsMode, PortForwardsScreenState};
use crate::config::Settings;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &mut PortForwardsScreenState, settings: &Settings) {
    match state.mode {
        PortForwardsMode::InstanceToggle => render_instance_toggle(f, state, settings),
        PortForwardsMode::GlobalEdit => render_global_edit(f, state, settings),
        PortForwardsMode::EditRule => render_edit_rule(f, state),
    }
}

fn render_instance_toggle(f: &mut Frame, state: &PortForwardsScreenState, settings: &Settings) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title_text = format!(
        "Port Forwarding — {} ({})",
        state.instance_name, state.instance_id
    );
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Rule list with checkboxes
    let enabled_aliases = settings
        .instance_port_forwards
        .get(&state.instance_id)
        .cloned()
        .unwrap_or_default();

    if settings.port_forwarding_rules.is_empty() {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled("No forwarding rules configured", Style::default().fg(Color::Gray))),
            Line::from(""),
            Line::from(Span::styled("Press 'r' to manage rules", Style::default().fg(Color::Yellow))),
        ];
        let empty = Paragraph::new(empty_lines)
            .block(Block::default().title("Rules").borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = settings
            .port_forwarding_rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let checked = enabled_aliases.contains(&rule.alias);
                let checkbox = if checked { "[✓]" } else { "[ ]" };
                let style = if i == state.selected_rule {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if checked {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };
                let text = format!(
                    "{}  {:<20} {} → {}",
                    checkbox, rule.alias, rule.local_port, rule.remote_port
                );
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Rules").borders(Borders::ALL));
        f.render_widget(list, chunks[1]);
    }

    // Help
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("[Space/Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Toggle  "),
        Span::styled("[r]", Style::default().fg(Color::Green)),
        Span::raw(" Manage rules  "),
        Span::styled("[Esc]", Style::default().fg(Color::Green)),
        Span::raw(" Back"),
    ])])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn render_global_edit(f: &mut Frame, state: &PortForwardsScreenState, settings: &Settings) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Port Forwarding Rules")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Rule list
    if settings.port_forwarding_rules.is_empty() {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled("No rules defined", Style::default().fg(Color::Gray))),
            Line::from(""),
            Line::from(Span::styled("Press 'a' to add a rule", Style::default().fg(Color::Yellow))),
        ];
        let empty = Paragraph::new(empty_lines)
            .block(Block::default().title("Rules").borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = settings
            .port_forwarding_rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let style = if i == state.selected_rule {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let text = format!(
                    "{}. {:<20} {} → {}",
                    i + 1,
                    rule.alias,
                    rule.local_port,
                    rule.remote_port
                );
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Rules").borders(Borders::ALL));
        f.render_widget(list, chunks[1]);
    }

    // Help
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("[a]", Style::default().fg(Color::Green)),
        Span::raw(" Add  "),
        Span::styled("[e]", Style::default().fg(Color::Green)),
        Span::raw(" Edit  "),
        Span::styled("[d]", Style::default().fg(Color::Green)),
        Span::raw(" Delete  "),
        Span::styled("[Esc]", Style::default().fg(Color::Green)),
        Span::raw(" Back"),
    ])])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn render_edit_rule(f: &mut Frame, state: &mut PortForwardsScreenState) {
    let edit = match state.edit_state {
        Some(ref mut e) => e,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(4)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Alias field
            Constraint::Length(5), // Local port field
            Constraint::Length(5), // Remote port field
            Constraint::Min(1),    // Spacer
            Constraint::Length(3), // Help
        ])
        .split(f.area());

    let title_text = if edit.edit_index.is_some() {
        "Edit Rule"
    } else {
        "Add Rule"
    };
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Highlight focused field border
    let focused_style = Style::default().fg(Color::Yellow);
    let normal_style = Style::default().fg(Color::Gray);

    let alias_style = if edit.focused_field == PortForwardField::Alias { focused_style } else { normal_style };
    let local_style = if edit.focused_field == PortForwardField::LocalPort { focused_style } else { normal_style };
    let remote_style = if edit.focused_field == PortForwardField::RemotePort { focused_style } else { normal_style };

    edit.alias.set_block(
        Block::default().title("Alias").borders(Borders::ALL).border_style(alias_style)
    );
    edit.local_port.set_block(
        Block::default().title("Local Port").borders(Borders::ALL).border_style(local_style)
    );
    edit.remote_port.set_block(
        Block::default().title("Remote Port").borders(Borders::ALL).border_style(remote_style)
    );

    f.render_widget(&edit.alias, chunks[1]);
    f.render_widget(&edit.local_port, chunks[2]);
    f.render_widget(&edit.remote_port, chunks[3]);

    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Green)),
        Span::raw(" Next field  "),
        Span::styled("[Shift+Tab]", Style::default().fg(Color::Green)),
        Span::raw(" Prev field  "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Green)),
        Span::raw(" Cancel"),
    ])])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[5]);
}
