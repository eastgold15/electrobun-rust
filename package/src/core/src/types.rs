//! Type definitions for Electrobun Core

use std::ffi::c_char;

/// Rectangle structure
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Point structure
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Display information
#[derive(Debug, Clone)]
pub struct Display {
    pub id: i64,
    pub bounds: Rect,
    pub work_area: Rect,
    pub scale_factor: f64,
    pub is_primary: bool,
}

/// Window style options
#[derive(Debug, Clone, Default)]
pub struct WindowStyle {
    pub borderless: bool,
    pub titled: bool,
    pub closable: bool,
    pub miniaturizable: bool,
    pub resizable: bool,
    pub unified_title_and_toolbar: bool,
    pub full_screen: bool,
    pub full_size_content_view: bool,
    pub utility_window: bool,
    pub doc_modal_window: bool,
    pub nonactivating_panel: bool,
    pub hud_window: bool,
}

/// Window options
#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub title: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub transparent: bool,
    pub hidden: bool,
    pub decorate: bool,
    pub resizable: bool,
    pub closable: bool,
    pub movable: bool,
    pub minimizable: bool,
    pub maximizable: bool,
    pub focusable: bool,
    pub always_on_top: bool,
    pub always_on_bottom: bool,
    pub fullscreen: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            transparent: false,
            hidden: false,
            decorate: true,
            resizable: true,
            closable: true,
            movable: true,
            minimizable: true,
            maximizable: true,
            focusable: true,
            always_on_top: false,
            always_on_bottom: false,
            fullscreen: false,
        }
    }
}

/// Window state (internal)
#[derive(Debug, Clone)]
pub struct WindowState {
    pub id: u32,
    pub title: String,
    pub bounds: Rect,
    pub transparent: bool,
    pub visible: bool,
    pub maximized: bool,
    pub minimized: bool,
    pub fullscreen: bool,
    // Callback handlers (stored as function pointers)
    pub close_handler: Option<Box<dyn Fn(u32) + Send + Sync>>,
    pub move_handler: Option<Box<dyn Fn(u32, f64, f64) + Send + Sync>>,
    pub resize_handler: Option<Box<dyn Fn(u32, f64, f64, f64, f64) + Send + Sync>>,
    pub focus_handler: Option<Box<dyn Fn(u32) + Send + Sync>>,
    pub blur_handler: Option<Box<dyn Fn(u32) + Send + Sync>>,
    pub key_handler: Option<Box<dyn Fn(u32, u32, u32, u32, u32) + Send + Sync>>,
}

/// Webview renderer type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebviewRendererKind {
    Native,
    Cef,
}

impl Default for WebviewRendererKind {
    fn default() -> Self {
        WebviewRendererKind::Native
    }
}

/// Webview options
#[derive(Debug, Clone)]
pub struct WebviewOptions {
    pub window_id: u32,
    pub url: String,
    pub secret_key: String,
    pub partition: String,
    pub sandboxed: bool,
    pub transparent: bool,
    pub visible: bool,
    pub bounds: Rect,
    pub renderer: WebviewRendererKind,
}

impl Default for WebviewOptions {
    fn default() -> Self {
        Self {
            window_id: 0,
            url: String::new(),
            secret_key: String::new(),
            partition: String::from("persist:default"),
            sandboxed: true,
            transparent: false,
            visible: true,
            bounds: Rect::default(),
            renderer: WebviewRendererKind::Native,
        }
    }
}

/// Webview state (internal)
#[derive(Debug, Clone)]
pub struct WebviewState {
    pub id: u32,
    pub window_id: u32,
    pub url: String,
    pub bounds: Rect,
    pub transparent: bool,
    pub visible: bool,
    pub renderer: WebviewRendererKind,
    pub secret_key: Vec<u8>,
}

/// WGPU view options
#[derive(Debug, Clone)]
pub struct WgpuViewOptions {
    pub window_id: u32,
    pub bounds: Rect,
    pub auto_resize: bool,
    pub start_transparent: bool,
    pub start_passthrough: bool,
}

impl Default for WgpuViewOptions {
    fn default() -> Self {
        Self {
            window_id: 0,
            bounds: Rect::default(),
            auto_resize: true,
            start_transparent: false,
            start_passthrough: false,
        }
    }
}

/// WGPU view state (internal)
#[derive(Debug, Clone)]
pub struct WgpuViewState {
    pub id: u32,
    pub window_id: u32,
    pub bounds: Rect,
    pub transparent: bool,
    pub visible: bool,
}

/// Tray options
#[derive(Debug, Clone)]
pub struct TrayOptions {
    pub title: String,
    pub image: String,
    pub is_template: bool,
    pub width: u32,
    pub height: u32,
}

impl Default for TrayOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            image: String::new(),
            is_template: false,
            width: 18,
            height: 18,
        }
    }
}

/// Tray state (internal)
#[derive(Debug, Clone)]
pub struct TrayState {
    pub id: u32,
    pub title: String,
    pub image: String,
    pub visible: bool,
    pub handler: Option<Box<dyn Fn(u32, &str) + Send + Sync>>,
}

/// Webview runtime state
#[derive(Debug, Clone, Default)]
pub struct WebviewRuntimeState {
    pub rpc_port: u32,
    pub preload_script: Option<String>,
    pub preload_script_sandboxed: Option<String>,
    pub configured: bool,
}

/// Host transport state
#[derive(Debug, Clone, Default)]
pub struct HostTransportState {
    pub started: bool,
    pub port: u32,
}

/// Default webview callbacks
#[derive(Debug, Clone, Default)]
pub struct DefaultWebviewCallbacks {
    pub navigation_callback: Option<
        std::sync::Arc<
            dyn Fn(u32, &str) -> u32 + Send + Sync,
        >,
    >,
    pub event_callback: Option<
        std::sync::Arc<
            dyn Fn(u32, &str, &str) + Send + Sync,
        >,
    >,
    pub bridge_callback: Option<
        std::sync::Arc<
            dyn Fn(u32, &str) + Send + Sync,
        >,
    >,
}

/// Pending host message
#[derive(Debug, Clone)]
pub struct PendingHostMessage {
    pub webview_id: u32,
    pub message: String,
}

/// App info structure
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub identifier: String,
    pub name: String,
    pub channel: String,
}

/// Paths structure
#[derive(Debug, Clone)]
pub struct Paths {
    pub home: String,
    pub app_data: String,
    pub config: String,
    pub cache: String,
    pub temp: String,
    pub logs: String,
    pub documents: String,
    pub downloads: String,
    pub desktop: String,
    pub pictures: String,
    pub music: String,
    pub videos: String,
    pub user_data: String,
    pub user_cache: String,
    pub user_logs: String,
}
