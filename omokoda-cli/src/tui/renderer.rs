use crate::tui::app::{App, AppState};
use crate::tui::widgets::{
    chat_view::render_chat, header::render_header, input_bar::render_input_bar,
    slash_menu::render_slash_menu, status_panel::render_status_panel,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(frame, app, vertical[0]);
    render_input_bar(frame, app, vertical[2]);

    let body_constraints: Vec<Constraint> = match (app.left_panel_visible, app.right_panel_visible)
    {
        (true, true) => vec![
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ],
        (true, false) => vec![Constraint::Percentage(25), Constraint::Percentage(75)],
        (false, true) => vec![Constraint::Percentage(75), Constraint::Percentage(25)],
        (false, false) => vec![Constraint::Percentage(100)],
    };

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(body_constraints)
        .split(vertical[1]);

    match (app.left_panel_visible, app.right_panel_visible) {
        (true, true) => {
            render_status_panel(frame, app, body[0]);
            render_chat(frame, app, body[1]);
            render_right_info(frame, app, body[2]);
        }
        (true, false) => {
            render_status_panel(frame, app, body[0]);
            render_chat(frame, app, body[1]);
        }
        (false, true) => {
            render_chat(frame, app, body[0]);
            render_right_info(frame, app, body[1]);
        }
        (false, false) => {
            render_chat(frame, app, body[0]);
        }
    }

    if app.state == AppState::SlashMenu {
        render_slash_menu(frame, app, area);
    }
    if app.state == AppState::Help {
        render_help(frame, area);
    }
}

fn render_right_info(frame: &mut Frame, app: &App, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            " Receipts",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  /receipts to view",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Memory",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  /memory to view",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Skills",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  /skills to browse",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Tools",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  /tools to catalog",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(" Tick {}", app.tick_count % 1000),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let panel = Paragraph::new(lines).block(
        Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(panel, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(56, 78, area);
    frame.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            " Keybindings",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  /          ", Style::default().fg(Color::Cyan)),
            Span::raw("Open slash command menu"),
        ]),
        Line::from(vec![
            Span::styled("  ?          ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C     ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("  q          ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit (when input empty)"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+L     ", Style::default().fg(Color::Cyan)),
            Span::raw("Clear chat"),
        ]),
        Line::from(vec![
            Span::styled("  F1         ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle left panel"),
        ]),
        Line::from(vec![
            Span::styled("  F2         ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle right panel"),
        ]),
        Line::from(vec![
            Span::styled("  Tab        ", Style::default().fg(Color::Cyan)),
            Span::raw("Cycle autocomplete in slash menu"),
        ]),
        Line::from(vec![
            Span::styled("  ↑ ↓        ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll chat / navigate menu"),
        ]),
        Line::from(vec![
            Span::styled("  Enter      ", Style::default().fg(Color::Cyan)),
            Span::raw("Submit / confirm selection"),
        ]),
        Line::from(vec![
            Span::styled("  Esc        ", Style::default().fg(Color::Cyan)),
            Span::raw("Close overlay"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press ? or Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(widget, popup);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}
