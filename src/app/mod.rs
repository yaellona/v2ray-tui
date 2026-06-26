mod event;

use crate::config::Agency;
use crate::proxy::ProxyNode;
use crate::singbox;
use crossterm::event::KeyCode;
use std::fs;
use std::path::Path;
use std::process::Child;

pub use event::poll_event;

#[derive(Debug)]
pub struct App {
    pub nodes: Vec<ProxyNode>,
    pub selected: usize,
    pub should_quit: bool,
    pub agencies: Vec<Agency>,
    pub proxy_running: bool,
    pub active_node: Option<usize>,
    child_process: Option<Child>,
}

impl App {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            selected: 0,
            should_quit: false,
            agencies: vec![],
            proxy_running: false,
            active_node: None,
            child_process: None,
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

    pub fn toggle_proxy(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        // 如果当前节点正在运行，且按的是同一个节点，则停止
        if self.proxy_running && self.active_node == Some(self.selected) {
            self.stop_proxy();
            return;
        }

        // 如果有其他节点在运行，先停止
        if self.proxy_running {
            self.stop_proxy();
        }

        // 启动新节点
        self.start_proxy();
    }

    fn start_proxy(&mut self) {
        let node = &self.nodes[self.selected];
        match singbox::start_proxy(node) {
            Ok(child) => {
                self.child_process = Some(child);
                self.proxy_running = true;
                self.active_node = Some(self.selected);
            }
            Err(e) => {
                eprintln!("启动代理失败: {}", e);
            }
        }
    }

    fn stop_proxy(&mut self) {
        if let Some(child) = &mut self.child_process {
            singbox::stop_proxy(child);
        }
        self.child_process = None;
        self.proxy_running = false;
        self.active_node = None;
    }

    pub fn cleanup(&mut self) {
        self.stop_proxy();
    }

    pub fn get_active_node_name(&self) -> Option<&str> {
        self.active_node
            .and_then(|idx| self.nodes.get(idx))
            .map(|node| node.name())
    }

    pub fn on_key(&mut self, key: KeyCode) {
        if key == KeyCode::Char('q') {
            self.should_quit = true;
            return;
        } else if self.nodes.is_empty() {
            return;
        }
        match key {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.nodes.len() - 1;
                }
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1) % self.nodes.len();
            }
            KeyCode::Enter => {
                self.toggle_proxy();
            }
            _ => {}
        }
    }
}
