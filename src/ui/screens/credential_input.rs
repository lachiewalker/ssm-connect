use crate::app::{CredentialField, CredentialInputState};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, state: &mut CredentialInputState, error: Option<&str>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Access Key
            Constraint::Length(3), // Secret Key
            Constraint::Length(3), // Session Token
            Constraint::Min(3),    // Help text
            Constraint::Length(3), // Hint message
            Constraint::Length(3), // Error
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("AWS Credential Input")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Access Key field
    let access_key_style = if state.focused_field == CredentialField::AccessKey {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let access_key_block = Block::default()
        .borders(Borders::ALL)
        .title("Access Key ID")
        .style(access_key_style);
    state.access_key.set_block(access_key_block);
    f.render_widget(&state.access_key, chunks[1]);

    // Secret Key field
    let secret_key_style = if state.focused_field == CredentialField::SecretKey {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let secret_key_block = Block::default()
        .borders(Borders::ALL)
        .title("Secret Access Key")
        .style(secret_key_style);
    state.secret_key.set_block(secret_key_block);
    f.render_widget(&state.secret_key, chunks[2]);

    // Session Token field
    let session_token_style = if state.focused_field == CredentialField::SessionToken {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let session_token_block = Block::default()
        .borders(Borders::ALL)
        .title("Session Token (optional)")
        .style(session_token_style);
    state.session_token.set_block(session_token_block);
    f.render_widget(&state.session_token, chunks[3]);

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(Color::Green)),
            Span::raw(" Switch fields  "),
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Submit  "),
            Span::styled("[Ctrl+D]", Style::default().fg(Color::Green)),
            Span::raw(" Default credentials"),
        ]),
        Line::from(vec![
            Span::styled("[Esc]", Style::default().fg(Color::Green)),
            Span::raw(" Exit  "),
            Span::styled("[Ctrl+Q]", Style::default().fg(Color::Green)),
            Span::raw(" Quit"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tip: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Paste AWS export block (all 3 lines) anywhere, or enter values in each field"),
        ]),
    ];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[4]);

    // Hint message (if any)
    if let Some(hint) = &state.hint_message {
        let hint_text = Paragraph::new(hint.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Hint"));
        f.render_widget(hint_text, chunks[5]);
    }

    // Error message
    if let Some(err) = error {
        let error_text = Paragraph::new(err)
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error_text, chunks[6]);
    }
}
