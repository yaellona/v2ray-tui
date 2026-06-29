mod action;
mod event;

use crate::config::Agency;
use crate::proxy::ProxyNode;
use crate::system_proxy;
use crossterm::event::KeyCode;
use std::process::Child;

pub use event::poll_event;

#[derive(Debug, PartialEq)]
pub enum PopupMode {
    None,
    UrlInput,
    AgencySelect,
}

#[derive(Debug)]
pub struct App {
    pub selected_node: usize,
    pub should_quit: bool,
    pub agencies: Vec<Agency>,
    pub proxy_running: bool,
    pub active_node: Option<usize>,
    pub(crate) child_process: Option<Child>,
    pub popup: PopupMode,
    pub url_input: String,
    pub selected_agency: usize,
    pub status_message: Option<String>,
    pub loading: bool,
    pub system_proxy_enabled: bool,
    pub viewing_all: bool,
    pub listen_port: u16,
}

impl App {
    pub fn new(agencies: Vec<Agency>) -> Self {
        let listen_port = crate::singbox::get_listen_port();
        Self {
            selected_node: 0,
            should_quit: false,
            agencies,
            proxy_running: false,
            active_node: None,
            child_process: None,
            popup: PopupMode::None,
            url_input: String::new(),
            selected_agency: 0,
            status_message: None,
            loading: false,
            system_proxy_enabled: system_proxy::get_system_proxy_status(listen_port),
            viewing_all: true,
            listen_port,
        }
    }

    pub fn current_agency(&self) -> Option<&Agency> {
        self.agencies.get(self.selected_agency)
    }

    pub fn current_nodes(&self) -> &[ProxyNode] {
        self.current_agency()
            .map(|a| a.nodes.as_slice())
            .unwrap_or_default()
    }

    pub fn current_node(&self) -> Option<&ProxyNode> {
        self.current_nodes().get(self.selected_node)
    }

    pub fn get_active_node_name(&self) -> Option<&str> {
        self.active_node
            .and_then(|idx| self.current_nodes().get(idx))
            .map(|node| node.name())
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match self.popup {
            PopupMode::UrlInput => {
                self.handle_url_input(key);
                return;
            }
            PopupMode::AgencySelect => {
                self.handle_agency_select(key);
                return;
            }
            PopupMode::None => {}
        }

        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('u') => {
                self.popup = PopupMode::UrlInput;
                self.url_input.clear();
            }
            KeyCode::Char('c') => {
                if !self.agencies.is_empty() {
                    self.popup = PopupMode::AgencySelect;
                    self.selected_agency = 0;
                }
            }
            KeyCode::Char('p') => {
                self.toggle_system_proxy();
            }
            KeyCode::Up => {
                let len = self.current_nodes().len();
                if len > 0 {
                    self.selected_node = if self.selected_node > 0 {
                        self.selected_node - 1
                    } else {
                        len - 1
                    };
                }
            }
            KeyCode::Down => {
                let len = self.current_nodes().len();
                if len > 0 {
                    self.selected_node = (self.selected_node + 1) % len;
                }
            }
            KeyCode::Enter => {
                self.toggle_proxy();
            }
            _ => {}
        }
    }
}
