use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Help content
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("SSM Connect - Help")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);

    // Help content
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("Instance List Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓ or j/k", Style::default().fg(Color::Green)),
            Span::raw("     Navigate through instance list"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green)),
            Span::raw("           Connect to selected instance via SSM"),
        ]),
        Line::from(vec![
            Span::styled("  s", Style::default().fg(Color::Green)),
            Span::raw("               Start a stopped instance"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Green)),
            Span::raw("               Change AWS region"),
        ]),
        Line::from(vec![
            Span::styled("  c", Style::default().fg(Color::Green)),
            Span::raw("               Configure auto-execute commands"),
        ]),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(Color::Green)),
            Span::raw("               Toggle this help screen"),
        ]),
        Line::from(vec![
            Span::styled("  q or Esc", Style::default().fg(Color::Green)),
            Span::raw("        Quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled("SSM Connection", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  To connect to an instance:"),
        Line::from("    1. The instance must be in 'Running' state"),
        Line::from("    2. The SSM agent must be 'Online' (✓)"),
        Line::from("    3. Press Enter to start an interactive session"),
        Line::from(""),
        Line::from("  SSM Status Indicators:"),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("✓", Style::default().fg(Color::Green)),
            Span::raw("  Online - Ready for connections"),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("⚠", Style::default().fg(Color::Yellow)),
            Span::raw("  Connection Lost - Agent offline"),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("✗", Style::default().fg(Color::Red)),
            Span::raw("  Not Installed - SSM agent not available"),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("?", Style::default().fg(Color::Gray)),
            Span::raw("  Unknown - Status unavailable"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Requirements", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("  - AWS CLI with Session Manager plugin installed"),
        Line::from("  - Valid AWS credentials with appropriate IAM permissions"),
        Line::from("  - EC2 instances with SSM agent installed and configured"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ? or Esc to return to instance list", Style::default().fg(Color::Gray)),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(help, chunks[1]);
}
