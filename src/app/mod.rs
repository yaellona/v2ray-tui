mod event;

use crate::config::{self, Agency};
use crate::proxy::ProxyNode;
use crate::singbox;
use crate::system::system_proxy;
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
    child_process: Option<Child>,
    pub popup: PopupMode,
    pub url_input: String,
    pub selected_agency: usize,
    pub status_message: Option<String>,
    pub loading: bool,
    pub system_proxy_enabled: bool,
    pub viewing_all: bool,
}
impl App {
    pub fn new(agencies: Vec<Agency>) -> Self {
        Self {
            selected_node: 0,
            should_quit: false,
            agencies: agencies,
            proxy_running: false,
            active_node: None,
            child_process: None,
            popup: PopupMode::None,
            url_input: String::new(),
            selected_agency: 0,
            status_message: None,
            loading: false,
            system_proxy_enabled: system_proxy::get_system_proxy_status(),
            viewing_all: true,
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

    pub fn switch_agency(&mut self, idx: usize) {
        if idx >= self.agencies.len() {
            return;
        }
        if self.proxy_running {
            self.stop_proxy();
        }
        self.selected_node = 0;
        self.active_node = None;
        self.selected_agency = idx;
        self.viewing_all = false;

        let provider = self
            .current_agency()
            .map(|a| a.provider.as_str())
            .unwrap_or("未知");
        self.status_message = Some(format!("已切换到: {}", provider));
    }

    pub fn fetch_and_add_agency(&mut self, url: &str) {
        if self.agencies.iter().any(|a| a.url == url) {
            self.status_message = Some("该订阅已存在".to_string());
            return;
        }

        self.loading = true;
        self.status_message = Some("正在拉取订阅...".to_string());

        match config::fetch_subscription(url) {
            Ok(agency) => {
                if let Err(e) = agency.save() {
                    self.status_message = Some(format!("保存订阅失败: {}", e));
                } else {
                    self.agencies.push(agency);
                    self.status_message = Some("订阅添加成功".to_string());
                }
            }
            Err(e) => {
                self.status_message = Some(format!("拉取失败: {}", e));
            }
        }

        self.loading = false;
    }

    pub fn toggle_proxy(&mut self) {
        if self.current_nodes().is_empty() {
            return;
        }
        if self.proxy_running && self.active_node == Some(self.selected_node) {
            self.stop_proxy();
            return;
        }
        if self.proxy_running {
            self.stop_proxy();
        }
        self.start_proxy();
    }

    fn start_proxy(&mut self) {
        let node = match self.current_node().cloned() {
            Some(node) => node,
            None => return,
        };
        match singbox::start_proxy(&node) {
            Ok(child) => {
                self.child_process = Some(child);
                self.proxy_running = true;
                self.active_node = Some(self.selected_node);
                self.status_message = Some(format!("已启动: {}", node.name()));
            }
            Err(e) => {
                self.status_message = Some(format!("启动代理失败: {}", e));
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

    pub fn toggle_system_proxy(&mut self) {
        let new_status = !self.system_proxy_enabled;
        match system_proxy::set_system_proxy(new_status) {
            Ok(()) => {
                self.system_proxy_enabled = new_status;
                self.status_message = Some(format!(
                    "系统代理已{}",
                    if new_status { "开启" } else { "关闭" }
                ));
            }
            Err(e) => {
                self.status_message = Some(format!("设置系统代理失败: {}", e));
            }
        }
    }

    pub fn get_active_node_name(&self) -> Option<&str> {
        self.active_node
            .and_then(|idx| self.current_nodes().get(idx))
            .map(|node| node.name())
    }

    fn handle_url_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.popup = PopupMode::None;
                self.url_input.clear();
            }
            KeyCode::Enter if !self.url_input.is_empty() => {
                let url = self.url_input.clone();
                self.popup = PopupMode::None;
                self.url_input.clear();
                self.fetch_and_add_agency(&url);
            }
            KeyCode::Backspace => {
                self.url_input.pop();
            }
            KeyCode::Char(c) => {
                self.url_input.push(c);
            }
            _ => {}
        }
    }

    fn handle_agency_select(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.popup = PopupMode::None;
            }
            KeyCode::Up => {
                if self.selected_agency > 0 {
                    self.selected_agency -= 1;
                } else {
                    self.selected_agency = self.agencies.len() - 1;
                }
            }
            KeyCode::Down => {
                self.selected_agency = (self.selected_agency + 1) % self.agencies.len();
            }
            KeyCode::Enter => {
                let idx = self.selected_agency;
                self.popup = PopupMode::None;
                self.switch_agency(idx);
            }
            _ => {}
        }
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
                return;
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
