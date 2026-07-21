use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect) {
    let help_text = vec![Line::from(vec![
        Span::styled("[↑/↓]", Style::default().fg(Color::Green)),
        Span::raw(" Navigate | "),
        Span::styled("[Enter]", Style::default().fg(Color::Green)),
        Span::raw(" Connect | "),
        Span::styled("[s]", Style::default().fg(Color::Green)),
        Span::raw(" Start | "),
        Span::styled("[r]", Style::default().fg(Color::Green)),
        Span::raw(" Region | "),
        Span::styled("[c]", Style::default().fg(Color::Green)),
        Span::raw(" Commands | "),
        Span::styled("[p]", Style::default().fg(Color::Green)),
        Span::raw(" Forwards | "),
        Span::styled("[?]", Style::default().fg(Color::Green)),
        Span::raw(" Help | "),
        Span::styled("[q]", Style::default().fg(Color::Green)),
        Span::raw(" Quit"),
    ])];

    let status = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status, area);
}
