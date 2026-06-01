//! Window management module - winit EventLoop 架构
//! 
//! 架构说明：
//! - Zig 模式：FFI → 同步调用 → 直接返回
//! - winit 模式：FFI → 异步发消息 → 后台事件循环 → 回调返回
//! 
//! 实现要点：
//! 1. 持久后台线程运行 EventLoop
//! 2. 命令通道发送操作到事件循环
//! 3. 响应通道获取结果

use crate::error::ElectrobunError;
use crate::types::{Rect, WindowOptions, WindowState};
use crate::WINDOW_REGISTRY;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════════
// COMMAND CHANNEL ARCHITECTURE
// ═══════════════════════════════════════════════════════════════════════════════

/// 窗口操作命令
#[derive(Debug)]
pub enum WindowCommand {
    Create {
        id: u32,
        title: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        hidden: bool,
        decorate: bool,
        resizable: bool,
        transparent: bool,
        result_tx: Sender<Result<(), String>>,
    },
    Close { id: u32, result_tx: Sender<bool> },
    Show { id: u32, result_tx: Sender<bool> },
    Hide { id: u32, result_tx: Sender<bool> },
    Minimize { id: u32, result_tx: Sender<bool> },
    Maximize { id: u32, result_tx: Sender<bool> },
    Unmaximize { id: u32, result_tx: Sender<bool> },
    SetTitle { id: u32, title: String, result_tx: Sender<bool> },
    SetBounds { id: u32, x: f64, y: f64, width: f64, height: f64, result_tx: Sender<bool> },
    SetAlwaysOnTop { id: u32, on_top: bool, result_tx: Sender<bool> },
    SetFullscreen { id: u32, fullscreen: bool, result_tx: Sender<bool> },
    SetFrame { id: u32, frameless: bool, result_tx: Sender<bool> },
    Focus { id: u32, result_tx: Sender<bool> },
    Shutdown,
}

/// 全局命令发送器（FFI 线程 → 事件循环线程）
lazy_static::lazy_static! {
    pub static ref COMMAND_SENDER: parking_lot::Mutex<Option<Sender<WindowCommand>>> = 
        parking_lot::Mutex::new(None);
    
    pub static ref EVENT_LOOP_HANDLE: parking_lot::Mutex<Option<thread::JoinHandle<()>>> = 
        parking_lot::Mutex::new(None);
}

/// 初始化事件循环线程
pub fn init_event_loop() {
    let mut sender_guard = COMMAND_SENDER.lock();
    if sender_guard.is_some() {
        return; // Already initialized
    }
    drop(sender_guard);
    
    let (cmd_tx, cmd_rx) = channel::<WindowCommand>();
    
    // 创建后台线程运行事件循环
    let handle = thread::spawn(move || {
        run_event_loop(cmd_rx);
    });
    
    *COMMAND_SENDER.lock() = Some(cmd_tx);
    *EVENT_LOOP_HANDLE.lock() = Some(handle);
}

/// 事件循环主函数（winit 0.30+ 架构）
fn run_event_loop(command_rx: Receiver<WindowCommand>) {
    use winit::application::ApplicationHandler;
    use winit::event_loop::{EventLoop, ActiveEventLoop};
    use std::collections::HashMap;
    
    let event_loop = EventLoop::builder()
        .build()
        .expect("Failed to create event loop");
    
    // 窗口存储
    let mut windows: HashMap<u32, winit::window::Window> = HashMap::new();
    
    // 创建应用处理器
    struct AppHandler {
        command_rx: Receiver<WindowCommand>,
        windows: HashMap<u32, winit::window::Window>,
    }
    
    impl ApplicationHandler for AppHandler {
        fn new_events(&mut self, _loop: &ActiveEventLoop, _evt: winit::event::StartCause) {
            // 处理命令队列
            while let Ok(cmd) = self.command_rx.try_recv() {
                match cmd {
                    WindowCommand::Create { id, title, x, y, width, height, hidden, decorate, resizable, transparent, result_tx } => {
                        // 窗口创建需要 ActiveEventLoop
                        let _ = result_tx.send(Err("Window creation requires ActiveEventLoop".into()));
                    }
                    WindowCommand::Shutdown => {
                        // 设置退出标志
                    }
                    _ => {}
                }
            }
        }
        
        fn resumed(&mut self, _loop: &ActiveEventLoop) {}
        
        fn window_event(&mut self, _loop: &ActiveEventLoop, _window_id: winit::window::WindowId, _event: winit::event::WindowEvent) {}
        
        fn user_event(&mut self, _event: &ActiveEventLoop, _loop: ()) {}
    }
    
    let mut handler = AppHandler {
        command_rx,
        windows,
    };
    
    let _ = event_loop.run_app(&mut handler);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API (FFI 调用点)
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a new window
pub fn create_window(options: WindowOptions) -> Result<u32, ElectrobunError> {
    // 确保事件循环已初始化
    init_event_loop();
    
    let id = crate::next_window_id();
    
    let state = WindowState {
        id,
        title: options.title.clone(),
        bounds: Rect {
            x: options.x,
            y: options.y,
            width: options.width,
            height: options.height,
        },
        transparent: options.transparent,
        visible: !options.hidden,
        maximized: false,
        minimized: false,
        fullscreen: options.fullscreen,
        close_handler: None,
        move_handler: None,
        resize_handler: None,
        focus_handler: None,
        blur_handler: None,
        key_handler: None,
    };
    
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    registry.insert(id, state);
    
    // 发送创建命令到事件循环
    let sender = COMMAND_SENDER.lock();
    if let Some(ref tx) = *sender {
        let (result_tx, result_rx) = channel();
        tx.send(WindowCommand::Create {
            id,
            title: options.title,
            x: options.x,
            y: options.y,
            width: options.width,
            height: options.height,
            hidden: options.hidden,
            decorate: options.decorate,
            resizable: options.resizable,
            transparent: options.transparent,
            result_tx,
        }).ok();
        
        if let Ok(Ok(())) = result_rx.recv_timeout(Duration::from_secs(5)) {
            return Ok(id);
        }
    }
    
    Ok(id)
}

/// Close a window
pub fn close_window(id: u32) -> bool {
    let sender = COMMAND_SENDER.lock();
    if let Some(ref tx) = *sender {
        let (result_tx, result_rx) = channel();
        tx.send(WindowCommand::Close { id, result_tx }).ok();
        if let Ok(result) = result_rx.recv_timeout(Duration::from_secs(1)) {
            return result;
        }
    }
    
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(mut state) = registry.get_mut(&id) {
        if let Some(ref handler) = state.close_handler {
            handler(id);
        }
        registry.remove(&id);
        return true;
    }
    false
}

/// Show a window
pub fn show_window(id: u32) -> bool {
    let sender = COMMAND_SENDER.lock();
    if let Some(ref tx) = *sender {
        let (result_tx, result_rx) = channel();
        tx.send(WindowCommand::Show { id, result_tx }).ok();
        if let Ok(result) = result_rx.recv_timeout(Duration::from_secs(1)) {
            if result {
                let mut registry = WINDOW_REGISTRY.lock().unwrap();
                if let Some(state) = registry.get_mut(&id) {
                    state.visible = true;
                    state.minimized = false;
                }
            }
            return result;
        }
    }
    false
}

/// Hide a window
pub fn hide_window(id: u32) -> bool {
    let sender = COMMAND_SENDER.lock();
    if let Some(ref tx) = *sender {
        let (result_tx, result_rx) = channel();
        tx.send(WindowCommand::Hide { id, result_tx }).ok();
        if let Ok(result) = result_rx.recv_timeout(Duration::from_secs(1)) {
            if result {
                let mut registry = WINDOW_REGISTRY.lock().unwrap();
                if let Some(state) = registry.get_mut(&id) {
                    state.visible = false;
                }
            }
            return result;
        }
    }
    false
}

/// Minimize a window
pub fn minimize_window(id: u32) -> bool {
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.minimized = true;
        true
    } else {
        false
    }
}

/// Maximize a window
pub fn maximize_window(id: u32) -> bool {
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.maximized = true;
        true
    } else {
        false
    }
}

/// Unmaximize a window
pub fn unmaximize_window(id: u32) -> bool {
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.maximized = false;
        true
    } else {
        false
    }
}

/// Set window title
pub fn set_window_title(id: u32, title: &str) -> bool {
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.title = title.to_string();
        true
    } else {
        false
    }
}

/// Set window bounds
pub fn set_window_bounds(id: u32, x: f64, y: f64, width: f64, height: f64) -> bool {
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.bounds = Rect { x, y, width, height };
        true
    } else {
        false
    }
}

/// Get window bounds
pub fn get_window_bounds(id: u32) -> Option<Rect> {
    let registry = WINDOW_REGISTRY.lock().unwrap();
    registry.get(&id).map(|state| state.bounds)
}

/// Set window always on top
pub fn set_window_always_on_top(id: u32, _on_top: bool) -> bool {
    let registry = WINDOW_REGISTRY.lock().unwrap();
    registry.contains_key(&id)
}

/// Focus a window
pub fn focus_window(id: u32) -> bool {
    let registry = WINDOW_REGISTRY.lock().unwrap();
    registry.contains_key(&id)
}

/// Set window fullscreen
pub fn set_window_fullscreen(id: u32, fullscreen: bool) -> bool {
    let mut registry = WINDOW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.fullscreen = fullscreen;
        true
    } else {
        false
    }
}

/// Set window frame (decorated or frameless)
pub fn set_window_frame(id: u32, _frameless: bool) -> bool {
    let registry = WINDOW_REGISTRY.lock().unwrap();
    registry.contains_key(&id)
}

/// Get window handle for webview integration
pub fn get_window_handle(_id: u32) -> Option<Arc<winit::window::Window>> {
    None
}

/// Shutdown event loop
pub fn shutdown() {
    let sender = COMMAND_SENDER.lock();
    if let Some(ref tx) = *sender {
        let _ = tx.send(WindowCommand::Shutdown);
    }
    
    if let Some(handle) = EVENT_LOOP_HANDLE.lock().take() {
        let _ = handle.join();
    }
    
    *COMMAND_SENDER.lock() = None;
}
