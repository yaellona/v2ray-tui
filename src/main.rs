mod app;
mod ui;
mod event;
mod components;
mod utils;
mod test;
use std::io;
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new();
    

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(key) = event::poll_event()? {
            app.on_key(key);
        }
        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}