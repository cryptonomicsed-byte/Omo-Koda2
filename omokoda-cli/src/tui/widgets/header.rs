use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ])
        .split(area);

    // Agent name + tier
    let stars: String = "★".repeat(app.tier as usize);
    let name_text = Line::from(vec![
        Span::styled(format!(" {stars} "), Style::default().fg(Color::Yellow)),
        Span::styled(
            app.agent_name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  Tier {}", app.tier),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let name_widget = Paragraph::new(name_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(name_widget, layout[0]);

    // Token gauge
    let pct = app.token_pct();
    let gauge_color = match pct {
        0..=60 => Color::Green,
        61..=85 => Color::Yellow,
        _ => Color::Red,
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Context ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(gauge_color))
        .percent(pct)
        .label(format!("{} / {} tokens", app.token_used, app.token_max));
    frame.render_widget(gauge, layout[1]);

    // Session ID
    let session_text = Line::from(vec![
        Span::styled(" Session: ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.session_id.clone(), Style::default().fg(Color::Cyan)),
    ]);
    let session_widget = Paragraph::new(session_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(session_widget, layout[2]);
}
