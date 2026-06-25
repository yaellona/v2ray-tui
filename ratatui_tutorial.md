# Ratatui 终端用户界面教程

## 目录
1. [简介](#简介)
2. [安装与设置](#安装与设置)
3. [基本概念](#基本概念)
4. [第一个TUI应用](#第一个tui应用)
5. [布局系统](#布局系统)
6. [常用组件](#常用组件)
7. [事件处理](#事件处理)
8. [状态管理](#状态管理)
9. [高级功能](#高级功能)
10. [最佳实践](#最佳实践)

## 简介

Ratatui 是一个用于构建终端用户界面(TUI)的 Rust 库。它是 `tui-rs` 的一个分支，提供了更活跃的开发和更多的功能。Ratatui 允许你创建丰富的终端应用程序，包括：

- 仪表板
- 系统监控工具
- 代码编辑器
- 游戏
- 交互式命令行工具

### 主要特性

- **跨平台**: 支持 Windows、macOS 和 Linux
- **高性能**: 使用双缓冲技术减少闪烁
- **丰富的组件**: 提供多种内置组件
- **灵活的布局**: 支持水平和垂直分割
- **事件驱动**: 支持键盘、鼠标事件

## 安装与设置

### 1. 创建新项目

```bash
cargo new my-tui-app
cd my-tui-app
```

### 2. 添加依赖

在 `Cargo.toml` 中添加以下依赖：

```toml
[dependencies]
ratatui = "0.25.0"
crossterm = "0.27.0"
```

### 3. 基本项目结构

```
my-tui-app/
├── Cargo.toml
└── src/
    └── main.rs
```

## 基本概念

### 终端后端

Ratatui 支持多种终端后端：
- **Crossterm**: 跨平台，推荐使用
- **Termion**: 仅 Unix 系统
- **Termwiz**: 跨平台，功能较少

### 缓冲区

Ratatui 使用双缓冲技术：
1. **前端缓冲区**: 用户看到的内容
2. **后端缓冲区**: 正在绘制的内容

绘制完成后，两个缓冲区交换，实现无闪烁更新。

### 组件

组件是 TUI 的基本构建块：
- **Block**: 带边框的容器
- **Paragraph**: 文本显示
- **List**: 列表显示
- **Table**: 表格显示
- **Gauge**: 进度条
- **Chart**: 图表
- **Tabs**: 标签页

## 第一个TUI应用

让我们创建一个简单的 "Hello World" TUI 应用：

```rust
use std::io;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

fn main() -> Result<(), io::Error> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 主循环
    loop {
        // 绘制UI
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("Hello Ratatui")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new("欢迎使用 Ratatui!")
                .block(block);
            f.render_widget(paragraph, size);
        })?;

        // 处理事件
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
```

## 布局系统

Ratatui 提供了灵活的布局系统，可以水平和垂直分割区域。

### 基本布局

```rust
use ratatui::layout::{Constraint, Direction, Layout};

// 创建垂直布局
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(30),  // 顶部 30%
        Constraint::Min(0),          // 剩余空间
    ])
    .split(f.size());

// 创建水平布局
let horizontal_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(50),  // 左侧 50%
        Constraint::Percentage(50),  // 右侧 50%
    ])
    .split(chunks[1]);
```

### 嵌套布局

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // 顶部固定高度
        Constraint::Min(0),     // 中间可变
        Constraint::Length(3),  // 底部固定高度
    ])
    .split(f.size());

// 中间区域再水平分割
let middle_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(70),
        Constraint::Percentage(30),
    ])
    .split(chunks[1]);
```

## 常用组件

### 1. Block（块）

Block 是最基本的容器，可以添加标题和边框：

```rust
use ratatui::widgets::{Block, Borders};

let block = Block::default()
    .title("标题")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Cyan))
    .style(Style::default().bg(Color::Black));
```

### 2. Paragraph（段落）

用于显示文本：

```rust
use ratatui::widgets::Paragraph;
use ratatui::text::{Span, Line};

// 简单文本
let paragraph = Paragraph::new("Hello World");

// 带样式的文本
let styled_text = vec![
    Line::from(vec![
        Span::styled("错误: ", Style::default().fg(Color::Red)),
        Span::raw("这是一条错误消息"),
    ]),
];

let paragraph = Paragraph::new(styled_text)
    .block(Block::default().title("消息").borders(Borders::ALL));
```

### 3. List（列表）

显示项目列表：

```rust
use ratatui::widgets::{List, ListItem};

let items = vec![
    ListItem::new("项目 1"),
    ListItem::new("项目 2"),
    ListItem::new("项目 3"),
];

let list = List::new(items)
    .block(Block::default().title("列表").borders(Borders::ALL))
    .highlight_style(Style::default().bg(Color::LightBlue))
    .highlight_symbol(">> ");
```

### 4. Table（表格）

显示表格数据：

```rust
use ratatui::widgets::{Table, Row, Cell};
use ratatui::layout::Constraint;

let rows = vec![
    Row::new(vec!["行1列1", "行1列2", "行1列3"]),
    Row::new(vec!["行2列1", "行2列2", "行2列3"]),
];

let widths = [
    Constraint::Length(10),
    Constraint::Length(15),
    Constraint::Length(20),
];

let table = Table::new(rows, widths)
    .block(Block::default().title("表格").borders(Borders::ALL))
    .header(Row::new(vec!["列1", "列2", "列3"]).style(Style::default().fg(Color::Yellow)));
```

### 5. Gauge（进度条）

显示进度：

```rust
use ratatui::widgets::Gauge;

let gauge = Gauge::default()
    .block(Block::default().title("进度").borders(Borders::ALL))
    .gauge_style(Style::default().fg(Color::Cyan))
    .ratio(0.75);  // 75% 进度
```

## 事件处理

Ratatui 使用 crossterm 处理终端事件。

### 键盘事件

```rust
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

if event::poll(std::time::Duration::from_millis(100))? {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('q') => {
                // 退出应用
                break;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+C 退出
                break;
            }
            KeyCode::Up => {
                // 处理上箭头
            }
            KeyCode::Down => {
                // 处理下箭头
            }
            _ => {}
        }
    }
}
```

### 鼠标事件

```rust
use crossterm::event::{MouseEvent, MouseEventKind};

if let Event::Mouse(mouse) = event::read()? {
    match mouse.kind {
        MouseEventKind::Down(btn) => {
            println!("鼠标按下: {:?}, 位置: ({}, {})", btn, mouse.column, mouse.row);
        }
        MouseEventKind::Moved => {
            println!("鼠标移动: ({}, {})", mouse.column, mouse.row);
        }
        _ => {}
    }
}
```

## 状态管理

对于复杂应用，需要管理应用状态：

```rust
use std::time::Instant;

#[derive(Debug)]
struct App {
    /// 应用状态
    items: Vec<String>,
    selected: usize,
    input: String,
    should_quit: bool,
    last_tick: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            items: vec!["项目1".to_string(), "项目2".to_string(), "项目3".to_string()],
            selected: 0,
            input: String::new(),
            should_quit: false,
            last_tick: Instant::now(),
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Backspace => { self.input.pop(); }
            _ => {}
        }
    }

    fn tick(&mut self) {
        // 定时更新逻辑
        self.last_tick = Instant::now();
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<(), io::Error> {
    loop {
        // 绘制UI
        terminal.draw(|f| ui(f, &app))?;

        // 处理事件
        let timeout = std::time::Duration::from_millis(100);
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        // 定时更新
        if app.last_tick.elapsed() >= std::time::Duration::from_secs(1) {
            app.tick();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // 顶部标题
    let title = Paragraph::new("我的 TUI 应用")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // 中间列表
    let items: Vec<ListItem> = app.items
        .iter()
        .map(|i| ListItem::new(i.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("列表").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightBlue))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut ListState::default().with_selected(Some(app.selected)));

    // 底部输入框
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title("输入").borders(Borders::ALL));
    f.render_widget(input, chunks[2]);
}
```

## 高级功能

### 自定义样式

```rust
use ratatui::style::{Color, Modifier, Style};

let style = Style::default()
    .fg(Color::White)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD | Modifier::ITALIC);
```

### 动态更新

使用定时器实现动态更新：

```rust
use std::time::{Duration, Instant};

let tick_rate = Duration::from_millis(250);
let mut last_tick = Instant::now();

loop {
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

    if event::poll(timeout)? {
        // 处理事件
    }

    if last_tick.elapsed() >= tick_rate {
        app.tick();
        last_tick = Instant::now();
    }
}
```

### 多线程

对于复杂应用，可以使用多线程：

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

// 事件处理线程
thread::spawn(move || {
    loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                tx.send(key).unwrap();
            }
        }
    }
});

// 主线程处理UI
loop {
    if let Ok(key) = rx.try_recv() {
        app.on_key(key);
    }
    // 绘制UI
}
```

## 最佳实践

### 1. 代码组织

```
src/
├── main.rs          # 入口点
├── app.rs           # 应用状态
├── ui.rs            # UI绘制
├── event.rs         # 事件处理
└── components/      # 自定义组件
    ├── mod.rs
    ├── header.rs
    └── footer.rs
```

### 2. 错误处理

```rust
use std::io;

#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Terminal(String),
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

type Result<T> = std::result::Result<T, AppError>;
```

### 3. 性能优化

- 使用 `terminal.draw()` 的闭包避免不必要的克隆
- 合理使用 `Constraint::Min` 和 `Constraint::Length`
- 避免在绘制循环中进行复杂计算

### 4. 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_ui() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new();

        terminal.draw(|f| ui(f, &app)).unwrap();

        // 验证输出
        let expected = "...";
        terminal.backend().assert_buffer_lines(expected.lines());
    }
}
```

## 示例项目

### 系统监控器

```rust
use sysinfo::{System, SystemExt};

struct SystemMonitor {
    system: System,
    cpu_usage: Vec<f64>,
    memory_usage: f64,
}

impl SystemMonitor {
    fn new() -> Self {
        Self {
            system: System::new_all(),
            cpu_usage: Vec::new(),
            memory_usage: 0.0,
        }
    }

    fn update(&mut self) {
        self.system.refresh_all();
        self.cpu_usage = self.system.cpus().iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .collect();
        self.memory_usage = self.system.used_memory() as f64 / 
                           self.system.total_memory() as f64 * 100.0;
    }
}
```

## 资源

- [Ratatui 官方文档](https://docs.rs/ratatui)
- [Ratatui GitHub](https://github.com/ratatui-org/ratatui)
- [Ratatui 示例](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [Crossterm 文档](https://docs.rs/crossterm)

## 总结

Ratatui 是一个功能强大且灵活的 TUI 库，适合构建各种终端应用程序。通过本教程，你已经学会了：

1. 如何设置和初始化 Ratatui
2. 使用布局系统组织界面
3. 创建和使用各种组件
4. 处理键盘和鼠标事件
5. 管理应用状态
6. 构建完整的 TUI 应用

现在你可以开始构建自己的终端应用程序了！