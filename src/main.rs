mod app;
mod config;
mod proxy;
mod singbox;
mod system;
mod ui;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new();
    app.read_config();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Some(key) = app::poll_event()? {
            app.on_key(key);
        }

        app.tick();

        if app.should_quit {
            break;
        }
    }

    // 清理子进程
    app.cleanup();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
