use crate::app::LoadingState;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render_overlay(f: &mut Frame, loading_state: &LoadingState) {
    let message = match loading_state {
        LoadingState::ValidatingCredentials => "Validating credentials...",
        LoadingState::LoadingInstances => "Loading instances...",
        LoadingState::Idle => return,
    };

    let area = tight_centered_rect(message, f.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let text = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(block);

    f.render_widget(Clear, area);
    f.render_widget(text, area);
}

/// Create a small centered rectangle that's just big enough for the text
fn tight_centered_rect(text: &str, r: Rect) -> Rect {
    // Calculate width: text length + 4 (for borders and padding)
    let width = (text.len() as u16 + 4).min(r.width - 4);
    // Fixed height: 3 lines (top border, text, bottom border)
    let height = 3;

    // Center the popup
    let x = (r.width.saturating_sub(width)) / 2;
    let y = (r.height.saturating_sub(height)) / 2;

    Rect {
        x: r.x + x,
        y: r.y + y,
        width,
        height,
    }
}
