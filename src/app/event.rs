use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub fn poll_event() -> io::Result<Option<KeyCode>> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(Some(key.code));
            }
        }
    }
    Ok(None)
}
