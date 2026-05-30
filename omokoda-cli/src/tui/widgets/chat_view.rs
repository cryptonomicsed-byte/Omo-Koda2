use crate::tui::app::{App, ChatMessage, MessageRole};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_chat(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .chat_messages
        .iter()
        .flat_map(message_to_lines)
        .collect();

    let total = lines.len();
    let height = area.height.saturating_sub(2) as usize;
    let scroll = if total > height {
        let max_scroll = total - height;
        app.chat_scroll.min(max_scroll) as u16
    } else {
        0
    };

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Chat ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(widget, area);
}

fn message_to_lines(msg: &ChatMessage) -> Vec<Line<'static>> {
    let (prefix_style, prefix) = match msg.role {
        MessageRole::User => (
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            "▶ You",
        ),
        MessageRole::Agent => (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            "★ Agent",
        ),
        MessageRole::System => (
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            "ℹ System",
        ),
        MessageRole::Tool => (Style::default().fg(Color::Green), "⚙ Tool"),
    };

    let mut result = vec![Line::from(Span::styled(prefix.to_string(), prefix_style))];

    for line in msg.content.lines() {
        result.push(Line::from(vec![
            Span::raw("  "),
            Span::raw(line.to_string()),
        ]));
    }
    result.push(Line::from(""));
    result
}
