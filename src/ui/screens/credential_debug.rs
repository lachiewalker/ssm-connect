use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Credential display
            Constraint::Length(4),  // Help text
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Debug: Parsed AWS Credentials")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Display credentials
    if let Some(creds) = &app.credentials {
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Access Key ID: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(&creds.access_key_id),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Secret Access Key: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(&creds.secret_access_key),
            ]),
            Line::from(""),
        ];

        if let Some(token) = &creds.session_token {
            lines.push(Line::from(vec![
                Span::styled("Session Token: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(token),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Session Token: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("(none)", Style::default().fg(Color::DarkGray)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Region: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(&app.region),
        ]));

        let cred_display = Paragraph::new(lines)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("Credentials"))
            .wrap(Wrap { trim: true });
        f.render_widget(cred_display, chunks[1]);
    }

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("[Esc]", Style::default().fg(Color::Green)),
            Span::raw(" Exit  "),
            Span::styled("[Ctrl+Q]", Style::default().fg(Color::Green)),
            Span::raw(" Quit"),
        ]),
    ];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(help, chunks[2]);
}
