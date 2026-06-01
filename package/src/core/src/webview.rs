//! Webview management module

use crate::error::ElectrobunError;
use crate::types::{Rect, WebviewRuntimeState, WebviewState, DefaultWebviewCallbacks, PendingHostMessage, WebviewRendererKind, WebviewOptions};
use crate::WEBVIEW_REGISTRY;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Global webview runtime state
lazy_static::lazy_static! {
    pub static ref WEBVIEW_RUNTIME_STATE: Arc<Mutex<WebviewRuntimeState>> = 
        Arc::new(Mutex::new(WebviewRuntimeState::default()));
}

/// Default webview callbacks
lazy_static::lazy_static! {
    pub static ref DEFAULT_WEBVIEW_CALLBACKS: Arc<Mutex<DefaultWebviewCallbacks>> = 
        Arc::new(Mutex::new(DefaultWebviewCallbacks::default()));
}

/// Pending host messages queue
lazy_static::lazy_static! {
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
    
    // Platform-specific webview creation
    #[cfg(target_os = "macos")]
    create_macos_webview(id, &options)?;
    
    #[cfg(target_os = "windows")]
    create_windows_webview(id, &options)?;
    
    #[cfg(target_os = "linux")]
    create_linux_webview(id, &options)?;
    
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
    
    if registry.remove(&id).is_some() {
        // Platform-specific cleanup
        #[cfg(target_os = "macos")]
        close_macos_webview(id);
        
        #[cfg(target_os = "windows")]
        close_windows_webview(id);
        
        #[cfg(target_os = "linux")]
        close_linux_webview(id);
        
        true
    } else {
        false
    }
}

/// Navigate webview to URL
pub fn webview_navigate(id: u32, url: &str) -> bool {
    let registry = WEBVIEW_REGISTRY.lock().unwrap();
    
    if let Some(state) = registry.get(&id) {
        // Store the new URL
        drop(registry);
        
        let mut registry = WEBVIEW_REGISTRY.lock().unwrap();
        if let Some(state) = registry.get_mut(&id) {
            state.url = url.to_string();
        }
        
        // Platform-specific navigation
        #[cfg(target_os = "macos")]
        navigate_macos_webview(id, url);
        
        #[cfg(target_os = "windows")]
        navigate_windows_webview(id, url);
        
        #[cfg(target_os = "linux")]
        navigate_linux_webview(id, url);
        
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
        
        #[cfg(target_os = "macos")]
        set_macos_webview_bounds(id, x, y, width, height);
        
        #[cfg(target_os = "windows")]
        set_windows_webview_bounds(id, x, y, width, height);
        
        #[cfg(target_os = "linux")]
        set_linux_webview_bounds(id, x, y, width, height);
        
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
        
        #[cfg(target_os = "macos")]
        set_macos_webview_visible(id, visible);
        
        #[cfg(target_os = "windows")]
        set_windows_webview_visible(id, visible);
        
        #[cfg(target_os = "linux")]
        set_linux_webview_visible(id, visible);
        
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
        
        #[cfg(target_os = "macos")]
        set_macos_webview_transparent(id, transparent);
        
        #[cfg(target_os = "windows")]
        set_windows_webview_transparent(id, transparent);
        
        #[cfg(target_os = "linux")]
        set_linux_webview_transparent(id, transparent);
        
        true
    } else {
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platform-specific implementations (stubs)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
fn create_macos_webview(_id: u32, _options: &WebviewOptions) -> Result<(), ElectrobunError> {
    // TODO: Implement with WKWebView
    Ok(())
}

#[cfg(target_os = "macos")]
fn close_macos_webview(_id: u32) {}

#[cfg(target_os = "macos")]
fn navigate_macos_webview(_id: u32, _url: &str) {}

#[cfg(target_os = "macos")]
fn set_macos_webview_bounds(_id: u32, _x: f64, _y: f64, _width: f64, _height: f64) {}

#[cfg(target_os = "macos")]
fn set_macos_webview_visible(_id: u32, _visible: bool) {}

#[cfg(target_os = "macos")]
fn set_macos_webview_transparent(_id: u32, _transparent: bool) {}

#[cfg(target_os = "windows")]
fn create_windows_webview(_id: u32, _options: &WebviewOptions) -> Result<(), ElectrobunError> {
    // TODO: Implement with WebView2
    Ok(())
}

#[cfg(target_os = "windows")]
fn close_windows_webview(_id: u32) {}

#[cfg(target_os = "windows")]
fn navigate_windows_webview(_id: u32, _url: &str) {}

#[cfg(target_os = "windows")]
fn set_windows_webview_bounds(_id: u32, _x: f64, _y: f64, _width: f64, _height: f64) {}

#[cfg(target_os = "windows")]
fn set_windows_webview_visible(_id: u32, _visible: bool) {}

#[cfg(target_os = "windows")]
fn set_windows_webview_transparent(_id: u32, _transparent: bool) {}

#[cfg(target_os = "linux")]
fn create_linux_webview(_id: u32, _options: &WebviewOptions) -> Result<(), ElectrobunError> {
    // TODO: Implement with WebKitGTK or CEF
    Ok(())
}

#[cfg(target_os = "linux")]
fn close_linux_webview(_id: u32) {}

#[cfg(target_os = "linux")]
fn navigate_linux_webview(_id: u32, _url: &str) {}

#[cfg(target_os = "linux")]
fn set_linux_webview_bounds(_id: u32, _x: f64, _y: f64, _width: f64, _height: f64) {}

#[cfg(target_os = "linux")]
fn set_linux_webview_visible(_id: u32, _visible: bool) {}

#[cfg(target_os = "linux")]
fn set_linux_webview_transparent(_id: u32, _transparent: bool) {}
