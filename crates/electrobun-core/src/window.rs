//! Window management module - winit EventLoop 架构
//!
//! All window/WebView objects live on the event loop thread.
//! Operations are sent via Command channel to that thread.
//!
//! 关键差异：
//! - EventLoop::run_app() 是无限阻塞的，后台线程一旦启动就不会退出
//! - 窗口/WebView 对象只能存在于事件循环线程上
//! - 所有窗口操作通过 Command channel 发送到事件循环线程
//! - 使用 oneshot channel 等待操作结果（模拟同步 FFI 调用）

use crate::error::ElectrobunError;
use crate::types::{Rect, WindowOptions, WindowState};
use crate::{WEBVIEW_REGISTRY, WINDOW_REGISTRY};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info};
use wry::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use wry::WebViewBuilder;

// ═══════════════════════════════════════════════════════════════════════════════
// COMMAND CHANNEL ARCHITECTURE
// ═══════════════════════════════════════════════════════════════════════════════

/// 窗口操作命令 —— FFI 线程发送到事件循环线程
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
    Close {
        id: u32,
        result_tx: Sender<bool>,
    },
    Show {
        id: u32,
        result_tx: Sender<bool>,
    },
    Hide {
        id: u32,
        result_tx: Sender<bool>,
    },
    Minimize {
        id: u32,
        result_tx: Sender<bool>,
    },
    Maximize {
        id: u32,
        result_tx: Sender<bool>,
    },
    Unmaximize {
        id: u32,
        result_tx: Sender<bool>,
    },
    SetTitle {
        id: u32,
        title: String,
        result_tx: Sender<bool>,
    },
    SetBounds {
        id: u32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        result_tx: Sender<bool>,
    },
    SetAlwaysOnTop {
        id: u32,
        on_top: bool,
        result_tx: Sender<bool>,
    },
    SetFullscreen {
        id: u32,
        fullscreen: bool,
        result_tx: Sender<bool>,
    },
    SetFrame {
        id: u32,
        frameless: bool,
        result_tx: Sender<bool>,
    },
    Focus {
        id: u32,
        result_tx: Sender<bool>,
    },
    Restore {
        id: u32,
        result_tx: Sender<bool>,
    },
    SetPosition {
        id: u32,
        x: f64,
        y: f64,
        result_tx: Sender<bool>,
    },
    SetSize {
        id: u32,
        width: f64,
        height: f64,
        result_tx: Sender<bool>,
    },
    IsMinimized {
        id: u32,
        result_tx: Sender<bool>,
    },
    IsMaximized {
        id: u32,
        result_tx: Sender<bool>,
    },
    IsFullscreen {
        id: u32,
        result_tx: Sender<bool>,
    },
    IsAlwaysOnTop {
        id: u32,
        result_tx: Sender<bool>,
    },
    // ── Webview commands ──
    CreateWebview {
        id: u32,
        window_id: u32,
        url: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        transparent: bool,
        preload_script: String,
        result_tx: Sender<Result<(), String>>,
    },
    NavigateWebview {
        id: u32,
        url: String,
        result_tx: Sender<bool>,
    },
    LoadHTMLWebview {
        id: u32,
        html: String,
        result_tx: Sender<bool>,
    },
    EvaluateJSWebview {
        id: u32,
        js: String,
        result_tx: Sender<bool>,
    },
    SetWebviewBounds {
        id: u32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        result_tx: Sender<bool>,
    },
    CloseWebview {
        id: u32,
        result_tx: Sender<bool>,
    },
    Shutdown,
}

/// 窗口事件回调类型
#[derive(Debug, Clone)]
pub enum WindowEventCallback {
    Close { id: u32 },
    Move { id: u32, x: f64, y: f64 },
    Resize { id: u32, width: f64, height: f64 },
    Focus { id: u32 },
    Blur { id: u32 },
}

/// 全局命令发送器（FFI 线程 → 事件循环线程）
lazy_static::lazy_static! {
    pub static ref COMMAND_SENDER: parking_lot::Mutex<Option<Sender<WindowCommand>>> =
        parking_lot::Mutex::new(None);

    pub static ref EVENT_LOOP_HANDLE: parking_lot::Mutex<Option<thread::JoinHandle<()>>> =
        parking_lot::Mutex::new(None);

    /// 事件回调通道（事件循环线程 → FFI 线程）
    pub static ref EVENT_CALLBACK_TX: parking_lot::Mutex<Option<Sender<WindowEventCallback>>> =
        parking_lot::Mutex::new(None);

    pub static ref EVENT_CALLBACK_RX: parking_lot::Mutex<Option<Receiver<WindowEventCallback>>> =
        parking_lot::Mutex::new(None);


    pub static ref COMMAND_RECEIVER: parking_lot::Mutex<Option<Receiver<WindowCommand>>> =
        parking_lot::Mutex::new(None);
}

/// 初始化事件循环通道（不启动线程；由 electrobun_core_run_main_thread 阻塞运行）（必须在第一次 FFI 调用前或首次调用时触发）
pub fn init_event_loop() {
    let mut sender_guard = COMMAND_SENDER.lock();
    if sender_guard.is_some() {
        return; // 已初始化
    }
    drop(sender_guard);

    let (cmd_tx, cmd_rx) = channel::<WindowCommand>();
    let (evt_tx, evt_rx) = channel::<WindowEventCallback>();

    *EVENT_CALLBACK_TX.lock() = Some(evt_tx);
    *EVENT_CALLBACK_RX.lock() = Some(evt_rx);
    *COMMAND_SENDER.lock() = Some(cmd_tx);
    // 注意：cmd_rx 直接传给事件循环线程，不存储在全局变量中
    
    // 自动在后台线程启动事件循环
    // 这样用户不需要手动调用 electrobun_core_run_main_thread
    thread::spawn(move || {
        info!("Starting window event loop in background thread");
        run_event_loop(cmd_rx);
    });
    
    // 给事件循环一点时间启动
    thread::sleep(Duration::from_millis(100));
}

/// 在主线程上阻塞运行事件循环（由 electrobun_core_run_main_thread 调用）
/// 
/// 注意：如果事件循环已经在后台线程运行，这个函数会直接返回。
/// 真正的窗口命令处理由后台事件循环线程负责。
pub fn run_blocking() {
    // 检查事件循环是否已经在运行
    let rx = COMMAND_RECEIVER.lock().take();
    if let Some(rx) = rx {
        // 事件循环还没启动，在主线程运行
        run_event_loop(rx);
        // run_event_loop 会一直运行直到收到 Shutdown 命令
    }
    // 如果事件循环已经在后台线程运行，直接返回
    // 后台线程会处理所有命令
}

/// 发送命令到事件循环线程并等待结果
pub fn send_command(cmd: WindowCommand, timeout: Duration) -> bool {
    let sender = COMMAND_SENDER.lock();
    if let Some(ref tx) = *sender {
        tx.send(cmd).ok();
        true
    } else {
        false
    }
}

/// 等待 bool 结果
pub fn wait_bool_result(rx: &Receiver<bool>, timeout: Duration) -> bool {
    rx.recv_timeout(timeout).unwrap_or(false)
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT LOOP (winit 0.30+ ApplicationHandler)
// ═══════════════════════════════════════════════════════════════════════════════

/// 事件循环主函数 —— 在后台线程中运行，阻塞直到 Shutdown
fn run_event_loop(command_rx: Receiver<WindowCommand>) {
    use winit::application::ApplicationHandler;
    use winit::event::{StartCause, WindowEvent as WinitWindowEvent};
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::{Window, WindowId};

    #[cfg(target_os = "windows")]
    use winit::platform::windows::EventLoopBuilderExtWindows;

    let mut builder = EventLoop::builder();
    #[cfg(target_os = "windows")]
    {
        builder.with_any_thread(true);
    }
    let event_loop = match builder.build() {
        Ok(el) => el,
        Err(e) => {
            error!("Failed to create event loop: {}", e);
            std::process::exit(1);
        },
    };

    // 预创建窗口映射：接收 window_id → Window
    // 如果窗口已经在 `run_app()` 之前创建就存入此表
    let precreated: std::sync::Mutex<HashMap<u32, Window>> = std::sync::Mutex::new(HashMap::new());
    drop(precreated); // TODO: 后续可以在此预创建窗口

    struct AppHandler {
        command_rx: Receiver<WindowCommand>,
        /// id → winit Window（只在事件循环线程内访问）
        windows: HashMap<u32, Window>,
        /// WindowId → 自定义 id（反向映射）
        window_ids: HashMap<WindowId, u32>,
        /// id → wry WebView（只在事件循环线程内访问）
        webviews: HashMap<u32, wry::WebView>,
        /// webview_id → window_id（窗口关闭时清理 webview）
        webview_window_map: HashMap<u32, u32>,
        /// 事件回调发送器
        event_tx: Option<Sender<WindowEventCallback>>,
        /// 退出标志
        exiting: bool,
        /// 事件循环是否已就绪（resumed 已被调用）
        ready: bool,
    }

    impl ApplicationHandler for AppHandler {
        fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
            // Poll 模式：没有 Windows 消息时也持续调用 new_events，
            // 确保 channel 里的命令能被及时处理（窗口创建前没有消息可泵）
            event_loop.set_control_flow(ControlFlow::Poll);

            // 每次事件循环迭代时，处理所有待处理命令
            while let Ok(cmd) = self.command_rx.try_recv() {
                if self.exiting {
                    break;
                }
                self.handle_command(event_loop, cmd);
            }
        }

        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            info!("Event loop resumed (ready for window creation)");
            self.ready = true;
            // resumed 后立即处理积压命令，包括 Create
            while let Ok(cmd) = self.command_rx.try_recv() {
                if self.exiting {
                    break;
                }
                self.handle_command(event_loop, cmd);
            }
        }

        fn window_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WinitWindowEvent,
        ) {
            // 将 winit 窗口事件转换为 Electrobun 回调
            let id = self.window_ids.get(&window_id).copied().unwrap_or(0);
            if id == 0 {
                return;
            }

            match event {
                WinitWindowEvent::CloseRequested => {
                    if let Some(ref tx) = self.event_tx {
                        let _ = tx.send(WindowEventCallback::Close { id });
                    }
                },
                WinitWindowEvent::Moved(pos) => {
                    if let Some(ref tx) = self.event_tx {
                        let _ = tx.send(WindowEventCallback::Move {
                            id,
                            x: pos.x as f64,
                            y: pos.y as f64,
                        });
                    }
                },
                WinitWindowEvent::Resized(size) => {
                    if let Some(ref tx) = self.event_tx {
                        let _ = tx.send(WindowEventCallback::Resize {
                            id,
                            width: size.width as f64,
                            height: size.height as f64,
                        });
                    }
                },
                WinitWindowEvent::Destroyed => {
                    self.windows.remove(&id);
                    self.window_ids.remove(&window_id);
                    // 清理关联的 webview（窗口关闭后 OS 已销毁子视图）
                    let orphaned: Vec<u32> = self
                        .webview_window_map
                        .iter()
                        .filter(|(_, &wid)| wid == id)
                        .map(|(&wvid, _)| wvid)
                        .collect();
                    for wvid in orphaned {
                        self.webviews.remove(&wvid);
                        self.webview_window_map.remove(&wvid);
                        WEBVIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).remove(&wvid);
                    }
                },
                WinitWindowEvent::Focused(focused) => {
                    if let Some(ref tx) = self.event_tx {
                        if focused {
                            let _ = tx.send(WindowEventCallback::Focus { id });
                        } else {
                            let _ = tx.send(WindowEventCallback::Blur { id });
                        }
                    }
                },
                _ => {},
            }
        }

        fn user_event(&mut self, _event: &ActiveEventLoop, _loop: ()) {}
    }

    impl AppHandler {
        /// 处理来自 FFI 线程的命令
        fn handle_command(&mut self, event_loop: &ActiveEventLoop, cmd: WindowCommand) {
            match cmd {
                WindowCommand::Create {
                    id,
                    title,
                    x,
                    y,
                    width,
                    height,
                    hidden,
                    decorate,
                    resizable,
                    transparent,
                    result_tx,
                } => {
                    let attrs = Window::default_attributes()
                        .with_title(&title)
                        .with_position(winit::dpi::LogicalPosition::new(x, y))
                        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
                        .with_visible(!hidden)
                        .with_decorations(decorate)
                        .with_resizable(resizable)
                        .with_transparent(transparent);

                    error!("Creating window on event loop thread (tid: {:?})", std::thread::current().id());
                    match event_loop.create_window(attrs) {
                        Ok(window) => {
                            let wid = window.id();
                            self.windows.insert(id, window);
                            self.window_ids.insert(wid, id);
                            let _ = result_tx.send(Ok(()));
                            info!("Window {} created successfully on event loop thread", id);
                        },
                        Err(e) => {
                            // Write to file for debugging
                            let _ = std::fs::write("C:\\Users\\boer\\winit_error.log", format!("{:?}", e));
                            let _ = result_tx.send(Err(format!("Failed to create window: {:?}", e)));
                        },
                    }
                },

                WindowCommand::Close { id, result_tx } => {
                    // 先销毁此窗口关联的所有 webview
                    let orphaned: Vec<u32> = self
                        .webview_window_map
                        .iter()
                        .filter(|(_, &wid)| wid == id)
                        .map(|(&wvid, _)| wvid)
                        .collect();
                    for wvid in orphaned {
                        self.webviews.remove(&wvid);
                        self.webview_window_map.remove(&wvid);
                        WEBVIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).remove(&wvid);
                    }
                    if let Some(window) = self.windows.remove(&id) {
                        let wid = window.id();
                        self.window_ids.remove(&wid);
                        drop(window); // 显式销毁
                        let _ = result_tx.send(true);
                    } else {
                        let _ = result_tx.send(false);
                    }
                },

                WindowCommand::Show { id, result_tx } => {
                    let result = self.with_window(id, |w| w.set_visible(true));
                    let _ = result_tx.send(result);
                },

                WindowCommand::Hide { id, result_tx } => {
                    let result = self.with_window(id, |w| w.set_visible(false));
                    let _ = result_tx.send(result);
                },

                WindowCommand::Minimize { id, result_tx } => {
                    let result = self.with_window(id, |w| w.set_minimized(true));
                    let _ = result_tx.send(result);
                },

                WindowCommand::Maximize { id, result_tx } => {
                    let result = self.with_window(id, |w| w.set_maximized(true));
                    let _ = result_tx.send(result);
                },

                WindowCommand::Unmaximize { id, result_tx } => {
                    let result = self.with_window(id, |w| w.set_maximized(false));
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetTitle {
                    id,
                    ref title,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| w.set_title(title));
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetBounds {
                    id,
                    x,
                    y,
                    width,
                    height,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| {
                        w.set_outer_position(winit::dpi::LogicalPosition::new(x, y));
                        let _ = w.request_inner_size(winit::dpi::LogicalSize::new(width, height));
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetAlwaysOnTop {
                    id,
                    on_top,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| {
                        w.set_window_level(if on_top {
                            winit::window::WindowLevel::AlwaysOnTop
                        } else {
                            winit::window::WindowLevel::Normal
                        });
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetFullscreen {
                    id,
                    fullscreen,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| {
                        if fullscreen {
                            w.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        } else {
                            w.set_fullscreen(None);
                        }
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetFrame {
                    id,
                    frameless,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| w.set_decorations(!frameless));
                    let _ = result_tx.send(result);
                },

                WindowCommand::Focus { id, result_tx } => {
                    let result = self.with_window(id, |w| w.focus_window());
                    let _ = result_tx.send(result);
                },

                WindowCommand::Restore { id, result_tx } => {
                    let result = self.with_window(id, |w| w.set_minimized(false));
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetPosition {
                    id,
                    x,
                    y,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| {
                        w.set_outer_position(winit::dpi::LogicalPosition::new(x, y));
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetSize {
                    id,
                    width,
                    height,
                    result_tx,
                } => {
                    let result = self.with_window(id, |w| {
                        let _ = w.request_inner_size(winit::dpi::LogicalSize::new(width, height));
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::IsMinimized { id, result_tx } => {
                    // winit doesn't have is_minimized() on Window, track from state
                    let result = self.windows.contains_key(&id);
                    let _ = result_tx.send(result);
                },

                WindowCommand::IsMaximized { id, result_tx } => {
                    let result = self
                        .windows
                        .get(&id)
                        .map(|w| w.is_maximized())
                        .unwrap_or(false);
                    let _ = result_tx.send(result);
                },

                WindowCommand::IsFullscreen { id, result_tx } => {
                    let result = self
                        .windows
                        .get(&id)
                        .map(|w| w.fullscreen().is_some())
                        .unwrap_or(false);
                    let _ = result_tx.send(result);
                },

                WindowCommand::IsAlwaysOnTop { id, result_tx } => {
                    let _ = result_tx.send(false);
                },

                // ── Webview commands ──
                WindowCommand::CreateWebview {
                    id,
                    window_id,
                    url,
                    x,
                    y,
                    width,
                    height,
                    transparent: _,
                    preload_script,
                    result_tx,
                } => {
                    if let Some(window) = self.windows.get(&window_id) {
                        let mut builder = WebViewBuilder::new().with_bounds(wry::Rect {
                            position: Position::Physical(PhysicalPosition::new(x as i32, y as i32)),
                            size: Size::Physical(PhysicalSize::new(width as u32, height as u32)),
                        });

                        // Register views:// custom protocol to serve local files
                        let views_root = crate::webview::WEBVIEW_RUNTIME_STATE.lock().unwrap_or_else(|e| e.into_inner())
                            .views_root
                            .clone()
                            .unwrap_or_default();
                        if !views_root.is_empty() {
                            let vr = views_root.clone();
                            use std::borrow::Cow;
                            use wry::http::Response;
                            use wry::WebViewId;
                            builder = builder.with_custom_protocol("views".into(), move |_id: WebViewId, request: wry::http::Request<Vec<u8>>| -> Response<Cow<'static, [u8]>> {
                                let uri = request.uri();
                                let path = if let Some(host) = uri.host() {
                                    format!("{}{}", host, uri.path())
                                } else {
                                    uri.path().trim_start_matches('/').to_string()
                                };
                                let file_path = format!("{}/{}", vr, path);
                                info!("[views://] URI: {}, path: {}, file: {}", uri, path, file_path);
                                match std::fs::read(&file_path) {
                                    Ok(data) => {
                                        let mime = mime_from_ext(&file_path);
                                        info!("[views://] OK: {} bytes, mime: {}", data.len(), mime);
                                        Response::builder()
                                            .header("Content-Type", mime)
                                            .body(Cow::Owned(data))
                                            .expect("valid Response")
                                    }
                                    Err(e) => {
                                        error!("[views://] ERROR: {} -> {}", file_path, e);
                                        Response::builder()
                                            .status(404)
                                            .body(Cow::Owned(Vec::new()))
                                            .expect("valid Response")
                                    }
                                }
                            });
                        }

                        if !url.is_empty() {
                            if url.starts_with("views://") && !views_root.is_empty() {
                                // Load views:// HTML directly via with_html, keeping custom
                                // protocol handler active for subresource requests (JS/CSS/images)
                                let file_path = url.trim_start_matches("views://");
                                let abs_path = format!("{}/{}", views_root, file_path);
                                if let Ok(html_content) = std::fs::read_to_string(&abs_path) {
                                    builder = builder.with_html(&html_content);
                                } else {
                                    builder = builder.with_url(&url);
                                }
                            } else {
                                builder = builder.with_url(&url);
                            }
                        }
                        if !preload_script.is_empty() {
                            builder = builder.with_initialization_script(&preload_script);
                        }

                        match builder.build_as_child(window) {
                            Ok(webview) => {
                                self.webviews.insert(id, webview);
                                self.webview_window_map.insert(id, window_id);
                                let _ = result_tx.send(Ok(()));
                            },
                            Err(e) => {
                                let _ =
                                    result_tx.send(Err(format!("WebView creation failed: {}", e)));
                            },
                        }
                    } else {
                        let _ = result_tx.send(Err("Window not found".to_string()));
                    }
                },

                WindowCommand::NavigateWebview { id, url, result_tx } => {
                    let result = self.with_webview(id, |wv| {
                        let _ = wv.load_url(&url);
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::LoadHTMLWebview {
                    id,
                    html,
                    result_tx,
                } => {
                    let result = self.with_webview(id, |wv| {
                        let _ = wv.load_html(&html);
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::EvaluateJSWebview { id, js, result_tx } => {
                    let result = self.with_webview(id, |wv| {
                        let _ = wv.evaluate_script(&js);
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::SetWebviewBounds {
                    id,
                    x,
                    y,
                    width,
                    height,
                    result_tx,
                } => {
                    let result = self.with_webview(id, |wv| {
                        let _ = wv.set_bounds(wry::Rect {
                            position: Position::Physical(PhysicalPosition::new(x as i32, y as i32)),
                            size: Size::Physical(PhysicalSize::new(width as u32, height as u32)),
                        });
                    });
                    let _ = result_tx.send(result);
                },

                WindowCommand::CloseWebview { id, result_tx } => {
                    let existed = self.webviews.remove(&id).is_some();
                    WEBVIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
                    let _ = result_tx.send(existed);
                },

                WindowCommand::Shutdown => {
                    self.exiting = true;
                    // 关闭所有窗口
                    self.webviews.clear();
                    self.windows.clear();
                    self.window_ids.clear();
                    // 退出事件循环
                    event_loop.exit();
                },
            }
        }

        /// 在指定 id 的窗口上执行操作
        fn with_window<F: FnOnce(&Window)>(&self, id: u32, f: F) -> bool {
            if let Some(window) = self.windows.get(&id) {
                f(window);
                true
            } else {
                false
            }
        }

        /// 在指定 id 的 webview 上执行操作
        fn with_webview<F: FnOnce(&wry::WebView)>(&self, id: u32, f: F) -> bool {
            if let Some(webview) = self.webviews.get(&id) {
                f(webview);
                true
            } else {
                false
            }
        }
    }

    // 获取事件回调发送器
    let event_tx = EVENT_CALLBACK_TX.lock().clone();

    let mut handler = AppHandler {
        command_rx,
        windows: HashMap::new(),
        window_ids: HashMap::new(),
        webviews: HashMap::new(),
        webview_window_map: HashMap::new(),
        event_tx,
        exiting: false,
        ready: false,
    };

    let _ = event_loop.run_app(&mut handler);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API (FFI 调用点)
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a new window
///
/// # Errors
///
/// Returns [`ElectrobunError`] if the window creation command fails.
pub fn create_window(options: WindowOptions) -> Result<u32, ElectrobunError> {
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

    {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        registry.insert(id, state);
    }

    // 发送创建命令到事件循环线程，等待结果
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Create {
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
    };

    if !send_command(cmd, Duration::from_secs(5)) {
        // 发送失败，但状态已注册，仍返回 id
        return Ok(id);
    }

    match result_rx.try_recv() {
        Ok(Ok(())) => Ok(id),
        Ok(Err(e)) => {
            WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
            Err(ElectrobunError::WindowOperationFailed(e))
        },
        _ => {
            // 事件循环尚未启动（主线程将在之后调用 run_blocking），
            // 先乐观返回 id，命令在 channel 中排队等待处理
            Ok(id)
        },
    }
}

/// Close a window
pub fn close_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Close { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mut state) = registry.get_mut(&id) {
            if let Some(ref handler) = state.close_handler {
                handler(id);
            }
            registry.remove(&id);
        }
    }
    result
}

/// Show a window
pub fn show_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Show { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.visible = true;
            state.minimized = false;
        }
    }
    result
}

/// Hide a window
pub fn hide_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Hide { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.visible = false;
        }
    }
    result
}

/// Minimize a window
pub fn minimize_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Minimize { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.minimized = true;
        }
    }
    result
}

/// Maximize a window
pub fn maximize_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Maximize { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.maximized = true;
        }
    }
    result
}

/// Unmaximize a window
pub fn unmaximize_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Unmaximize { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.maximized = false;
        }
    }
    result
}

/// Set window title
pub fn set_window_title(id: u32, title: &str) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetTitle {
        id,
        title: title.to_string(),
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.title = title.to_string();
        }
    }
    result
}

/// Set window bounds
pub fn set_window_bounds(id: u32, x: f64, y: f64, width: f64, height: f64) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetBounds {
        id,
        x,
        y,
        width,
        height,
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.bounds = Rect {
                x,
                y,
                width,
                height,
            };
        }
    }
    result
}

/// Get window bounds
pub fn get_window_bounds(id: u32) -> Option<Rect> {
    let registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.get(&id).map(|state| state.bounds)
}

/// Set window always on top
pub fn set_window_always_on_top(id: u32, on_top: bool) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetAlwaysOnTop {
        id,
        on_top,
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    wait_bool_result(&result_rx, Duration::from_secs(1))
}

/// Focus a window
pub fn focus_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Focus { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    wait_bool_result(&result_rx, Duration::from_secs(1))
}

/// Set window fullscreen
pub fn set_window_fullscreen(id: u32, fullscreen: bool) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetFullscreen {
        id,
        fullscreen,
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.fullscreen = fullscreen;
        }
    }
    result
}

/// Set window frame (decorated or frameless)
pub fn set_window_frame(id: u32, frameless: bool) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetFrame {
        id,
        frameless,
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    wait_bool_result(&result_rx, Duration::from_secs(1))
}

/// Restore a window (from minimized)
pub fn restore_window(id: u32) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::Restore { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.minimized = false;
        }
    }
    result
}

/// Set window position
pub fn set_window_position(id: u32, x: f64, y: f64) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetPosition {
        id,
        x,
        y,
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.bounds.x = x;
            state.bounds.y = y;
        }
    }
    result
}

/// Set window size
pub fn set_window_size(id: u32, width: f64, height: f64) -> bool {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::SetSize {
        id,
        width,
        height,
        result_tx,
    };
    send_command(cmd, Duration::from_secs(1));
    let result = wait_bool_result(&result_rx, Duration::from_secs(1));

    if result {
        let mut registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = registry.get_mut(&id) {
            state.bounds.width = width;
            state.bounds.height = height;
        }
    }
    result
}

/// Check if window is minimized
pub fn is_window_minimized(id: u32) -> Option<bool> {
    let registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.get(&id).map(|state| state.minimized)
}

/// Check if window is maximized
pub fn is_window_maximized(id: u32) -> Option<bool> {
    let registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.get(&id).map(|state| state.maximized)
}

/// Check if window is fullscreen
pub fn is_window_fullscreen(id: u32) -> bool {
    let registry = WINDOW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry
        .get(&id)
        .map(|state| state.fullscreen)
        .unwrap_or(false)
}

/// Check if window is always on top
pub fn is_window_always_on_top(id: u32) -> Option<bool> {
    let (result_tx, result_rx) = channel();
    let cmd = WindowCommand::IsAlwaysOnTop { id, result_tx };
    send_command(cmd, Duration::from_secs(1));
    Some(wait_bool_result(&result_rx, Duration::from_secs(1)))
}

/// Get window handle for webview integration (stub — 需要在事件循环线程内获取)
pub fn get_window_handle(_id: u32) -> Option<Arc<winit::window::Window>> {
    // 注意：winit Window 不能跨线程使用
    // wry 的 build_as_child 需要在事件循环线程内调用
    None
}

/// 轮询窗口事件回调（FFI 层定期调用）
pub fn poll_events() -> Vec<WindowEventCallback> {
    let mut events = Vec::new();
    let rx_guard = EVENT_CALLBACK_RX.lock();
    if let Some(ref rx) = *rx_guard {
        while let Ok(evt) = rx.try_recv() {
            events.push(evt);
        }
    }
    events
}

/// Shutdown event loop
pub fn shutdown() {
    let (result_tx, _result_rx) = channel::<bool>();
    let cmd = WindowCommand::Shutdown;
    send_command(cmd, Duration::from_secs(2));

    if let Some(handle) = EVENT_LOOP_HANDLE.lock().take() {
        let _ = handle.join();
    }

    *COMMAND_SENDER.lock() = None;
    *EVENT_CALLBACK_TX.lock() = None;
    *EVENT_CALLBACK_RX.lock() = None;
}

fn mime_from_ext(path: &str) -> &'static str {
    if path.ends_with(".html") || path.ends_with(".htm") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}
