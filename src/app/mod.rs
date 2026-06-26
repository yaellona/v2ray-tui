mod event;

use crate::config::{self, Agency};
use crate::proxy::ProxyNode;
use crate::singbox;
use crate::system::system_proxy;
use crossterm::event::KeyCode;
use dirs::config_dir;
use std::fs;
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
    pub nodes: Vec<ProxyNode>,
    pub selected: usize,
    pub should_quit: bool,
    pub agencies: Vec<Agency>,
    pub proxy_running: bool,
    pub active_node: Option<usize>,
    child_process: Option<Child>,
    pub popup: PopupMode,
    pub url_input: String,
    pub agency_selected: usize,
    pub status_message: Option<String>,
    pub loading: bool,
    pub system_proxy_enabled: bool,
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
            popup: PopupMode::None,
            url_input: String::new(),
            agency_selected: 0,
            status_message: None,
            loading: false,
            system_proxy_enabled: system_proxy::get_system_proxy_status(),
        }
    }

    pub fn readconfig(&mut self) {
        let config_dir = config_dir().unwrap().join("ladderust");
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

    pub fn refresh_nodes(&mut self) {
        self.nodes.clear();
        for agency in &self.agencies {
            for node in &agency.node {
                self.nodes.push(node.clone());
            }
        }
        self.selected = 0;
    }

    pub fn switch_agency(&mut self, idx: usize) {
        if idx >= self.agencies.len() {
            return;
        }

        // 如果代理正在运行，先停止
        if self.proxy_running {
            self.stop_proxy();
        }

        self.nodes.clear();
        let agency = &self.agencies[idx];
        for node in &agency.node {
            self.nodes.push(node.clone());
        }
        self.selected = 0;
        self.agency_selected = idx;

        let provider = agency
            .info
            .as_ref()
            .and_then(|i| i.provider.as_deref())
            .unwrap_or("未知");
        self.status_message = Some(format!("已切换到: {}", provider));
    }

    pub fn fetch_and_add_agency(&mut self, url: &str) {
        self.loading = true;
        self.status_message = Some("正在拉取订阅...".to_string());

        match config::fetch_subscription(url) {
            Ok(agency) => {
                let provider = agency
                    .info
                    .as_ref()
                    .and_then(|i| i.provider.as_deref())
                    .unwrap_or("未知")
                    .to_string();
                let node_count = agency.node.len();

                // 保存到配置文件
                if let Err(e) = agency.save_to_config() {
                    self.status_message = Some(format!("保存失败: {}", e));
                    self.loading = false;
                    return;
                }

                self.agencies.push(agency);
                self.refresh_nodes();
                self.status_message = Some(format!("已添加: {} ({} 个节点)", provider, node_count));
            }
            Err(e) => {
                self.status_message = Some(format!("拉取失败: {}", e));
            }
        }
        self.loading = false;
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
            .and_then(|idx| self.nodes.get(idx))
            .map(|node| node.name())
    }

    fn handle_url_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.popup = PopupMode::None;
                self.url_input.clear();
            }
            KeyCode::Enter => {
                if !self.url_input.is_empty() {
                    let url = self.url_input.clone();
                    self.popup = PopupMode::None;
                    self.url_input.clear();
                    self.fetch_and_add_agency(&url);
                }
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
                if self.agency_selected > 0 {
                    self.agency_selected -= 1;
                } else {
                    self.agency_selected = self.agencies.len() - 1;
                }
            }
            KeyCode::Down => {
                self.agency_selected = (self.agency_selected + 1) % self.agencies.len();
            }
            KeyCode::Enter => {
                let idx = self.agency_selected;
                self.popup = PopupMode::None;
                self.switch_agency(idx);
            }
            _ => {}
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        // 弹窗模式下处理弹窗按键
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

        // 正常模式
        if key == KeyCode::Char('q') {
            self.should_quit = true;
            return;
        }

        match key {
            KeyCode::Char('u') => {
                self.popup = PopupMode::UrlInput;
                self.url_input.clear();
            }
            KeyCode::Char('c') => {
                if !self.agencies.is_empty() {
                    self.popup = PopupMode::AgencySelect;
                    self.agency_selected = 0;
                }
            }
            KeyCode::Char('s') => {
                // 切换显示所有节点或当前代理商节点
                self.refresh_nodes();
                self.status_message = Some("已刷新节点列表".to_string());
            }
            KeyCode::Char('p') => {
                self.toggle_system_proxy();
            }
            _ => {
                if self.nodes.is_empty() {
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
    }
}
