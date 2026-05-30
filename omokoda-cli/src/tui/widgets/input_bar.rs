use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_input_bar(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(34)])
        .split(area);

    let input_val = app.input.value();
    let cursor_pos = app.input.visual_cursor();

    let input_line = Line::from(vec![
        Span::styled(
            " ▶ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(input_val.to_string()),
        Span::styled("▌", Style::default().fg(Color::Yellow)),
    ]);

    let input_widget = Paragraph::new(input_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    frame.render_widget(input_widget, layout[0]);

    // Position cursor (3 = " ▶ " prefix, 1 = border)
    let cursor_x = layout[0].x + 1 + 3 + cursor_pos as u16;
    let cursor_y = layout[0].y + 1;
    if cursor_x < layout[0].x + layout[0].width - 1 && cursor_y < layout[0].y + layout[0].height - 1
    {
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    // Hint bar
    let hints = Line::from(vec![
        Span::styled(" Tab", Style::default().fg(Color::DarkGray)),
        Span::raw(" complete  "),
        Span::styled("?", Style::default().fg(Color::DarkGray)),
        Span::raw(" help  "),
        Span::styled("/", Style::default().fg(Color::DarkGray)),
        Span::raw(" cmds  "),
        Span::styled("q", Style::default().fg(Color::DarkGray)),
        Span::raw(" quit"),
    ]);
    let hint_widget = Paragraph::new(hints).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(hint_widget, layout[1]);
}
