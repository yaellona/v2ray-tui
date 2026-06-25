use crate::utils::{Agency, NodeItem};
use crossterm::event::KeyCode;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct App {
    pub nodes: Vec<NodeItem>,
    pub selected: usize,
    pub should_quit: bool,
    pub agencies: Vec<Agency>,
}

impl App {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            selected: 0,
            should_quit: false,
            agencies: vec![],
        }
    }

    pub fn readconfig(&mut self) {
        let config_dir = Path::new("./config");
        if !config_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(mut agency) = serde_json::from_str::<Agency>(&content) {
                            for node in agency.node.drain(..) {
                                self.nodes.push(node);
                            }
                            self.agencies.push(agency);
                        }
                    }
                }
            }
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up => {
                if self.nodes.len() == 0 {
                    self.selected = 0;
                } else if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.nodes.len() - 1;
                }
            }
            KeyCode::Down => {
                if self.nodes.len() == 0 {
                    self.selected = 0;
                } else {
                    self.selected = (self.selected + 1) % self.nodes.len();
                }
            }
            KeyCode::Enter => {
                if self.nodes.len() == 0 {
                    return;
                }
            }
            _ => {}
        }
    }
}
