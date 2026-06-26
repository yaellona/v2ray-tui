mod event;

use crate::config::{self, Agency};
use crate::proxy::ProxyNode;
use crate::singbox;
use crate::system::system_proxy;
use crossterm::event::KeyCode;
use dirs::config_dir;
use std::fs;
use std::process::Child;
use std::sync::mpsc;

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
    pub viewing_all: bool,
    fetch_rx: Option<mpsc::Receiver<Result<Agency, String>>>,
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
            viewing_all: true,
            fetch_rx: None,
        }
    }

    pub fn read_config(&mut self) {
        let config_dir = config_dir().unwrap().join("ladderust");
        if !config_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json")
                    && let Ok(content) = fs::read_to_string(&path)
                    && let Ok(agency) = serde_json::from_str::<Agency>(&content)
                {
                    self.agencies.push(agency);
                }
            }
        }

        self.refresh_nodes();
        self.viewing_all = true;
    }

    pub fn refresh_nodes(&mut self) {
        if self.proxy_running {
            self.stop_proxy();
        }
        self.nodes.clear();
        for agency in &self.agencies {
            for node in &agency.nodes {
                self.nodes.push(node.clone());
            }
        }
        self.selected = 0;
        self.active_node = None;
        self.viewing_all = true;
    }

    pub fn switch_agency(&mut self, idx: usize) {
        if idx >= self.agencies.len() {
            return;
        }

        if self.proxy_running {
            self.stop_proxy();
        }

        self.nodes.clear();
        let agency = &self.agencies[idx];
        for node in &agency.nodes {
            self.nodes.push(node.clone());
        }
        self.selected = 0;
        self.active_node = None;
        self.agency_selected = idx;
        self.viewing_all = false;

        let provider = agency
            .info
            .as_ref()
            .and_then(|i| i.provider.as_deref())
            .unwrap_or("未知");
        self.status_message = Some(format!("已切换到: {}", provider));
    }

    pub fn fetch_and_add_agency(&mut self, url: &str) {
        // 检查是否已存在相同 URL 的订阅
        if self.agencies.iter().any(|a| a.url == url) {
            self.status_message = Some("该订阅已存在".to_string());
            return;
        }

        self.loading = true;
        self.status_message = Some("正在拉取订阅...".to_string());

        let url_owned = url.to_string();
        let (tx, rx) = mpsc::channel();
        self.fetch_rx = Some(rx);

        std::thread::spawn(move || {
            let result = config::fetch_subscription(&url_owned)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    pub fn check_fetch_result(&mut self) {
        let rx = match &self.fetch_rx {
            Some(rx) => rx,
            None => return,
        };

        match rx.try_recv() {
            Ok(result) => {
                self.fetch_rx = None;
                self.loading = false;
                match result {
                    Ok(agency) => {
                        let provider = agency
                            .info
                            .as_ref()
                            .and_then(|i| i.provider.as_deref())
                            .unwrap_or("未知")
                            .to_string();
                        let node_count = agency.nodes.len();

                        if let Err(e) = agency.save_to_config() {
                            self.status_message = Some(format!("保存失败: {}", e));
                            return;
                        }

                        self.agencies.push(agency);
                        self.refresh_nodes();
                        self.status_message =
                            Some(format!("已添加: {} ({} 个节点)", provider, node_count));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("拉取失败: {}", e));
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.fetch_rx = None;
                self.loading = false;
                self.status_message = Some("拉取订阅失败: 连接断开".to_string());
            }
        }
    }

    pub fn check_process_health(&mut self) {
        if !self.proxy_running {
            return;
        }

        if let Some(child) = &mut self.child_process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let msg = if status.success() {
                        "sing-box 进程已退出".to_string()
                    } else {
                        format!("sing-box 进程异常退出 (状态: {})", status)
                    };
                    self.child_process = None;
                    self.proxy_running = false;
                    self.active_node = None;
                    self.status_message = Some(msg);
                }
                Ok(None) => {}
                Err(_) => {
                    self.child_process = None;
                    self.proxy_running = false;
                    self.active_node = None;
                    self.status_message = Some("检查 sing-box 进程状态失败".to_string());
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.check_fetch_result();
        self.check_process_health();
    }

    pub fn toggle_proxy(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        if self.proxy_running && self.active_node == Some(self.selected) {
            self.stop_proxy();
            return;
        }

        if self.proxy_running {
            self.stop_proxy();
        }

        self.start_proxy();
    }

    fn start_proxy(&mut self) {
        let node = &self.nodes[self.selected];
        match singbox::start_proxy(node) {
            Ok(child) => {
                self.child_process = Some(child);
                self.proxy_running = true;
                self.active_node = Some(self.selected);
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
            .and_then(|idx| self.nodes.get(idx))
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
