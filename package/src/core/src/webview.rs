//! Webview management module - wry WebView 封装
//!
//! 架构说明：
//! - wry WebView 必须在事件循环线程中创建（build_as_child 需要 &Window）
//! - 通过 WindowCommand 通道发送创建请求
//! - IPC handler 将 JS→Rust 消息转发到 PENDING_HOST_MESSAGES 队列

use crate::error::ElectrobunError;
use crate::types::{Rect, WebviewRuntimeState, WebviewState, DefaultWebviewCallbacks, PendingHostMessage, WebviewOptions};
use crate::WEBVIEW_REGISTRY;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Global webview runtime state
lazy_static::lazy_static! {
    pub static ref WEBVIEW_RUNTIME_STATE: Arc<Mutex<WebviewRuntimeState>> =
        Arc::new(Mutex::new(WebviewRuntimeState::default()));

    pub static ref DEFAULT_WEBVIEW_CALLBACKS: Arc<Mutex<DefaultWebviewCallbacks>> =
        Arc::new(Mutex::new(DefaultWebviewCallbacks::default()));

    pub static ref PENDING_HOST_MESSAGES: Arc<Mutex<VecDeque<PendingHostMessage>>> =
        Arc::new(Mutex::new(VecDeque::new()));
}

/// Create a new webview
pub fn create_webview(options: WebviewOptions) -> Result<u32, ElectrobunError> {
    let id = crate::next_webview_id();

    let secret_key = parse_secret_key(&options.secret_key)?;

    let state = WebviewState {
        id,
        window_id: options.window_id,
        url: options.url.clone(),
        bounds: options.bounds,
        transparent: options.transparent,
        visible: options.visible,
        renderer: options.renderer,
        secret_key,
    };

    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.insert(id, state);

    // TODO: 将 WebView 创建命令发送到事件循环线程
    // 需要在 AppHandler::handle_command 中处理 CreateWebview 命令
    // 使用 wry::WebViewBuilder::new().build_as_child(&window)

    Ok(id)
}

/// Parse the secret key from comma-separated bytes
fn parse_secret_key(key_str: &str) -> Result<Vec<u8>, ElectrobunError> {
    if key_str.is_empty() {
        return Ok(vec![0u8; 32]);
    }

    let mut key = Vec::with_capacity(32);
    for part in key_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let byte = trimmed.parse::<u8>()
            .map_err(|_| ElectrobunError::CryptoError(format!("Invalid secret key byte: {}", trimmed)))?;
        key.push(byte);
    }

    if key.len() != 32 {
        return Err(ElectrobunError::CryptoError(format!(
            "Secret key must be exactly 32 bytes, got {}", key.len()
        )));
    }

    Ok(key)
}

/// Close a webview
pub fn close_webview(id: u32) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.remove(&id).is_some()
}

/// Navigate webview to URL
pub fn webview_navigate(id: u32, url: &str) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();

    if let Some(state) = registry.get_mut(&id) {
        state.url = url.to_string();
        true
    } else {
        false
    }
}

/// Set webview bounds
pub fn set_webview_bounds(id: u32, x: f64, y: f64, width: f64, height: f64) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();

    if let Some(state) = registry.get_mut(&id) {
        state.bounds = Rect { x, y, width, height };
        true
    } else {
        false
    }
}

/// Get webview bounds
pub fn get_webview_bounds(id: u32) -> Option<Rect> {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).map(|state| state.bounds)
}

/// Show/hide webview
pub fn set_webview_visible(id: u32, visible: bool) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();

    if let Some(state) = registry.get_mut(&id) {
        state.visible = visible;
        true
    } else {
        false
    }
}

/// Set webview transparent
pub fn set_webview_transparent(id: u32, transparent: bool) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();

    if let Some(state) = registry.get_mut(&id) {
        state.transparent = transparent;
        true
    } else {
        false
    }
}

/// Enqueue a pending host message (JS → Rust IPC)
pub fn enqueue_pending_host_message(webview_id: u32, message: String) {
    let pending = PendingHostMessage {
        webview_id,
        message,
    };

    let mut queue = PENDING_HOST_MESSAGES.lock().unwrap();
    queue.push_back(pending);
}

/// Pop the next queued message (Rust → JS)
pub fn pop_next_message() -> Option<(u32, String)> {
    let mut queue = PENDING_HOST_MESSAGES.lock().unwrap();

    if let Some(msg) = queue.pop_front() {
        Some((msg.webview_id, msg.message))
    } else {
        None
    }
}

/// Send message to webview (from host to webview)
pub fn send_message_to_webview(webview_id: u32, message: &str) -> bool {
    enqueue_pending_host_message(webview_id, message.to_string());
    true
}
