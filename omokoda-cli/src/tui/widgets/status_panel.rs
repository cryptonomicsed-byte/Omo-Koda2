use crate::tui::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_status_panel(frame: &mut Frame, app: &App, area: Rect) {
    let tier = app.tier as usize;
    let filled: String = "★".repeat(tier.min(7));
    let empty: String = "☆".repeat(7usize.saturating_sub(tier));

    let lines = vec![
        Line::from(Span::styled(
            " Agent",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("  {}", app.agent_name)),
        Line::from(""),
        Line::from(Span::styled(
            " Tier",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(filled, Style::default().fg(Color::Yellow)),
            Span::styled(empty, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Session",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("  {}", app.session_id)),
        Line::from(""),
        Line::from(Span::styled(
            " Context",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("  {}%", app.token_pct())),
        Line::from(format!("  {} / {}", app.token_used, app.token_max)),
        Line::from(""),
        Line::from(Span::styled(
            " Keys",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  F1 left  F2 right",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  / cmds   ? help",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Ctrl+L clear",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let panel = Paragraph::new(lines).block(
        Block::default()
            .title(" Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(panel, area);
}
