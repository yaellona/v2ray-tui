use super::{App, PopupMode};
use crate::config;
use crate::singbox;
use crate::system_proxy;
use crossterm::event::KeyCode;

impl App {
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
        match system_proxy::set_system_proxy(new_status, self.listen_port) {
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

    pub(super) fn handle_url_input(&mut self, key: KeyCode) {
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

    pub(super) fn handle_agency_select(&mut self, key: KeyCode) {
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
}
