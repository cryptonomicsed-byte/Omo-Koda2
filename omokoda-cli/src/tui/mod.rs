pub mod app;
pub mod events;
pub mod renderer;
pub mod widgets;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub use app::App;
pub use events::{AppEvent, EventHandler};

pub fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let events = EventHandler::new(250);

    let result = loop {
        terminal.draw(|frame| renderer::render(frame, &mut app))?;

        match events.next()? {
            AppEvent::Key(key) => {
                if app.handle_key(key) {
                    break Ok(());
                }
            }
            AppEvent::Tick => {
                app.on_tick();
            }
        }

        if app.should_quit {
            break Ok(());
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
