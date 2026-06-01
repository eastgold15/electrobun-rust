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
        html_content: None,
        passthrough: false,
        navigation_rules: None,
        preload_script: String::new(),
        devtools_open: false,
        zoom: 1.0,
        can_go_back: false,
        can_go_forward: false,
        navigation_history: vec![options.url.clone()],
        navigation_position: 0,
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

/// Load HTML content directly into the webview
pub fn load_html(id: u32, html: &str) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.html_content = Some(html.to_string());
        true
    } else {
        false
    }
}

/// Check if webview can go back in navigation history
pub fn webview_can_go_back(id: u32) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).map(|s| s.navigation_position > 0).unwrap_or(false)
}

/// Check if webview can go forward in navigation history
pub fn webview_can_go_forward(id: u32) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get(&id) {
        state.navigation_position + 1 < state.navigation_history.len()
    } else {
        false
    }
}

/// Navigate back in history
pub fn webview_go_back(id: u32) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        if state.navigation_position > 0 {
            state.navigation_position -= 1;
            if let Some(url) = state.navigation_history.get(state.navigation_position) {
                state.url = url.clone();
                return true;
            }
        }
    }
    false
}

/// Navigate forward in history
pub fn webview_go_forward(id: u32) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        let next = state.navigation_position + 1;
        if next < state.navigation_history.len() {
            state.navigation_position = next;
            if let Some(url) = state.navigation_history.get(next) {
                state.url = url.clone();
                return true;
            }
        }
    }
    false
}

/// Reload the webview
pub fn webview_reload(id: u32) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Stop loading
pub fn webview_stop(id: u32) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Set webview HTML content (for custom rendering)
pub fn set_webview_html_content(id: u32, html: &str) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.html_content = Some(html.to_string());
        true
    } else {
        false
    }
}

/// Set webview passthrough (click-through) mode
pub fn webview_set_passthrough(id: u32, passthrough: bool) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.passthrough = passthrough;
        true
    } else {
        false
    }
}

/// Set webview hidden
pub fn webview_set_hidden(id: u32, hidden: bool) -> bool {
    set_webview_visible(id, !hidden)
}

/// Set navigation rules (URL filtering)
pub fn set_navigation_rules(id: u32, rules_json: &str) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.navigation_rules = Some(rules_json.to_string());
        true
    } else {
        false
    }
}

/// Find in page
pub fn find_in_page(id: u32, _search: &str, _forward: bool, _find_next: bool) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Stop find in page
pub fn stop_find(id: u32, _clear_selection: bool) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Evaluate JavaScript in the webview
pub fn evaluate_javascript(id: u32, js: &str) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        // Store the JS for evaluation when connected
        state.html_content = Some(js.to_string()); // placeholder
        true
    } else {
        false
    }
}

/// Dispatch an event to the webview's host bridge
pub fn dispatch_host_webview_event(id: u32, _event_name: &str, _detail: &str) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Clear the webview's host transport
pub fn clear_host_transport(id: u32) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Send an internal message to the webview
pub fn send_internal_message(id: u32, _message: &str) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).is_some()
}

/// Update the preload script for the webview
pub fn update_preload_script(id: u32, preload: &str) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.preload_script = preload.to_string();
        true
    } else {
        false
    }
}

/// Open DevTools for the webview
pub fn open_devtools(id: u32) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.devtools_open = true;
        true
    } else {
        false
    }
}

/// Close DevTools for the webview
pub fn close_devtools(id: u32) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.devtools_open = false;
        true
    } else {
        false
    }
}

/// Toggle DevTools for the webview
pub fn toggle_devtools(id: u32) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.devtools_open = !state.devtools_open;
        true
    } else {
        false
    }
}

/// Set page zoom level
pub fn set_page_zoom(id: u32, zoom: f64) -> bool {
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    if let Some(state) = registry.get_mut(&id) {
        state.zoom = zoom;
        true
    } else {
        false
    }
}

/// Get page zoom level
pub fn get_page_zoom(id: u32) -> f64 {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.get(&id).map(|s| s.zoom).unwrap_or(1.0)
}
