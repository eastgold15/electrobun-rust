//! Webview management module - wry WebView 封装
//!
//! 架构说明：
//! - 使用 wry 库进行跨平台 WebView 管理
//! - wry 内部使用各平台的 WebView 实现 (WKWebView/WebView2/WebKitGTK)

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
    // Get next available ID
    let id = crate::next_webview_id();
    
    // Parse secret key
    let secret_key = parse_secret_key(&options.secret_key)?;
    
    // Create webview state
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
    
    // Register webview
    let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
    registry.insert(id, state);
    
    // Platform-specific webview creation will be implemented with wry
    // For now, just return the ID
    
    Ok(id)
}

/// Parse the secret key from comma-separated bytes
fn parse_secret_key(key_str: &str) -> Result<Vec<u8>, ElectrobunError> {
    if key_str.is_empty() {
        return Ok(vec![0u8; 32]); // Default key
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

/// Enqueue a pending host message
pub fn enqueue_pending_host_message(webview_id: u32, message: String) {
    let pending = PendingHostMessage {
        webview_id,
        message,
    };
    
    let mut queue = PENDING_HOST_MESSAGES.lock().unwrap();
    queue.push_back(pending);
}

/// Pop the next queued message
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

// ═══════════════════════════════════════════════════════════════════════════════
// WRY WEBVIEW BUILDER (Integration with winit)
// ═══════════════════════════════════════════════════════════════════════════════

/// Create wry WebView with winit window
/// This function should be called from the event loop thread
#[cfg(feature = "webview")]
pub fn create_wry_webview(
    window: &winit::window::Window,
    options: &WebviewOptions,
) -> Result<wry::WebView, ElectrobunError> {
    use wry::WebViewBuilder;
    
    let mut builder = WebViewBuilder::new()
        .with_url(&options.url)
        .with_bounds(wry::dpi::LogicalRect {
            x: options.bounds.x,
            y: options.bounds.y,
            width: options.bounds.width,
            height: options.bounds.height,
        })
        .with_transparent(options.transparent)
        .with_visible(options.visible);
    
    // Add IPC handler
    builder = builder.with_ipc_handler(|webview_id, message| {
        enqueue_pending_host_message(webview_id as u32, message.to_string());
    });
    
    // Build as child of winit window
    let webview = builder
        .build_as_child(window)
        .map_err(|e| ElectrobunError::WebviewOperationFailed(e.to_string()))?;
    
    Ok(webview)
}

#[cfg(not(feature = "webview"))]
pub fn create_wry_webview(
    _window: &winit::window::Window,
    _options: &WebviewOptions,
) -> Result<(), ElectrobunError> {
    Err(ElectrobunError::InitializationFailed("WebView feature not enabled".into()))
}
