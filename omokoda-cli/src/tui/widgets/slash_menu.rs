use crate::tui::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn render_slash_menu(frame: &mut Frame, app: &App, area: Rect) {
    let cmds = app.filtered_commands();
    if cmds.is_empty() {
        return;
    }

    let visible = (cmds.len() as u16).min(14);
    let height = visible + 2; // borders
    let popup = slash_popup_rect(area, height);
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = cmds
        .iter()
        .map(|&&(cmd, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{cmd:<18}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.slash_selected));

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Commands  ↑↓ navigate · Enter select · Esc close ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, popup, &mut state);
}

fn slash_popup_rect(area: Rect, height: u16) -> Rect {
    let width = area.width.min(64);
    let x = area.x + 1;
    // Place just above the input bar (last 3 rows) with a 1-row gap
    let bottom = area.y + area.height;
    let input_bar_height = 3u16;
    let y = bottom
        .saturating_sub(input_bar_height)
        .saturating_sub(height);
    Rect::new(x, y.max(area.y), width, height)
}
