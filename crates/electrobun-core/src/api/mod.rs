/**
 * Electrobun Core API
 *
 * 使用 #[eden_ipc] 宏定义 FFI API
 */

use electrobun_macros::{eden_ipc, eden_stream};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

// ============================================================================
// Window Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowParams {
    pub width: u32,
    pub height: u32,
    pub title: String,
    #[serde(default)]
    pub resizable: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub frameless: bool,
    pub url: Option<String>,
    pub html: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub resizable: bool,
    pub transparent: bool,
    pub frameless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowError {
    pub message: String,
}

// ============================================================================
// App Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub message: String,
}

// ============================================================================
// WebView Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewOptions {
    pub url: Option<String>,
    pub html: Option<String>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewError {
    pub message: String,
}

// ============================================================================
// API Traits
// ============================================================================

#[eden_ipc]
pub trait WindowAPI {
    fn create_window(&self, params: WindowParams) -> Result<Window, WindowError>;
    fn close_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn show_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn hide_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn minimize_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn maximize_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn restore_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn focus_window(&self, window_id: u32) -> Result<(), WindowError>;
    fn set_window_title(&self, window_id: u32, title: String) -> Result<(), WindowError>;
    fn set_window_size(&self, window_id: u32, width: u32, height: u32) -> Result<(), WindowError>;
    fn set_window_position(&self, window_id: u32, x: f64, y: f64) -> Result<(), WindowError>;
    fn set_window_fullscreen(&self, window_id: u32, fullscreen: bool) -> Result<(), WindowError>;
    fn set_window_always_on_top(&self, window_id: u32, on_top: bool) -> Result<(), WindowError>;
    fn set_window_frame(&self, window_id: u32, frameless: bool) -> Result<(), WindowError>;
    fn get_window_bounds(&self, window_id: u32) -> Result<WindowBounds, WindowError>;
}

#[eden_ipc]
pub trait AppAPI {
    fn get_app_name(&self) -> String;
    fn get_app_version(&self) -> String;
    fn get_app_data_path(&self) -> String;
    fn quit(&self) -> Result<(), AppError>;
    fn open_external(&self, url: String) -> Result<(), AppError>;
    fn show_item_in_folder(&self, path: String) -> Result<(), AppError>;
}

#[eden_ipc]
pub trait WebViewAPI {
    fn create_webview(&self, options: WebViewOptions) -> Result<u32, WebViewError>;
    fn close_webview(&self, webview_id: u32) -> Result<(), WebViewError>;
    fn navigate(&self, webview_id: u32, url: String) -> Result<(), WebViewError>;
    fn navigate_back(&self, webview_id: u32) -> Result<(), WebViewError>;
    fn navigate_forward(&self, webview_id: u32) -> Result<(), WebViewError>;
    fn reload(&self, webview_id: u32) -> Result<(), WebViewError>;
    fn load_html(&self, webview_id: u32, html: String) -> Result<(), WebViewError>;
    fn set_webview_bounds(&self, webview_id: u32, bounds: WebViewBounds) -> Result<(), WebViewError>;
    fn set_webview_visible(&self, webview_id: u32, visible: bool) -> Result<(), WebViewError>;
    fn resize(&self, webview_id: u32, width: f64, height: f64) -> Result<(), WebViewError>;
    fn send_message(&self, webview_id: u32, message: String) -> Result<(), WebViewError>;
}

// ============================================================================
// Implementations
// ============================================================================

pub struct ElectrobunApp;

impl WindowAPI for ElectrobunApp {
    fn create_window(&self, params: WindowParams) -> Result<Window, WindowError> {
        let opts = crate::types::WindowOptions {
            title: params.title.clone(),
            width: params.width as f64,
            height: params.height as f64,
            transparent: params.transparent,
            resizable: params.resizable,
            decorate: !params.frameless,
            x: 0.0, y: 0.0, hidden: false,
            closable: true, movable: true, minimizable: true, maximizable: true, focusable: true,
            always_on_top: false, always_on_bottom: false, fullscreen: false,
        };
        match crate::window::create_window(opts) {
            Ok(id) => Ok(Window {
                id, title: params.title.clone(),
                width: params.width, height: params.height,
                x: 0, y: 0, resizable: params.resizable,
                transparent: params.transparent, frameless: params.frameless,
            }),
            Err(e) => Err(WindowError { message: e.to_string() }),
        }
    }

    fn close_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::close_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn show_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::show_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn hide_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::hide_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn minimize_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::minimize_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn maximize_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::maximize_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn restore_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::restore_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn focus_window(&self, window_id: u32) -> Result<(), WindowError> {
        if crate::window::focus_window(window_id) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn set_window_title(&self, window_id: u32, title: String) -> Result<(), WindowError> {
        if crate::window::set_window_title(window_id, &title) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn set_window_size(&self, window_id: u32, width: u32, height: u32) -> Result<(), WindowError> {
        if crate::window::set_window_size(window_id, width as f64, height as f64) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn set_window_position(&self, window_id: u32, x: f64, y: f64) -> Result<(), WindowError> {
        if crate::window::set_window_position(window_id, x, y) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn set_window_fullscreen(&self, window_id: u32, fullscreen: bool) -> Result<(), WindowError> {
        if crate::window::set_window_fullscreen(window_id, fullscreen) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn set_window_always_on_top(&self, window_id: u32, on_top: bool) -> Result<(), WindowError> {
        if crate::window::set_window_always_on_top(window_id, on_top) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn set_window_frame(&self, window_id: u32, frameless: bool) -> Result<(), WindowError> {
        if crate::window::set_window_frame(window_id, frameless) { Ok(()) }
        else { Err(WindowError { message: format!("Window {} not found", window_id) }) }
    }

    fn get_window_bounds(&self, window_id: u32) -> Result<WindowBounds, WindowError> {
        match crate::window::get_window_bounds(window_id) {
            Some(r) => Ok(WindowBounds { x: r.x, y: r.y, width: r.width, height: r.height }),
            None => Err(WindowError { message: format!("Window {} not found", window_id) }),
        }
    }
}

impl AppAPI for ElectrobunApp {
    fn get_app_name(&self) -> String { "Electrobun".to_string() }
    fn get_app_version(&self) -> String { env!("CARGO_PKG_VERSION").to_string() }
    fn get_app_data_path(&self) -> String {
        std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()
    }
    fn quit(&self) -> Result<(), AppError> { std::process::exit(0); }
    fn open_external(&self, url: String) -> Result<(), AppError> {
        match open::that(&url) {
            Ok(()) => Ok(()),
            Err(e) => Err(AppError { message: format!("Failed to open: {}", e) }),
        }
    }
    fn show_item_in_folder(&self, path: String) -> Result<(), AppError> {
        match crate::dialog::show_item_in_folder(&path) {
            Ok(()) => Ok(()),
            Err(e) => Err(AppError { message: e.to_string() }),
        }
    }
}

impl WebViewAPI for ElectrobunApp {
    fn create_webview(&self, options: WebViewOptions) -> Result<u32, WebViewError> {
        let opts = crate::types::WebviewOptions {
            window_id: 0,
            url: options.url.unwrap_or_default(),
            secret_key: String::new(),
            partition: String::from("persist:default"),
            sandboxed: true, use_wry: true, transparent: false, visible: true,
            bounds: crate::types::Rect {
                x: options.x.unwrap_or(0) as f64, y: options.y.unwrap_or(0) as f64,
                width: options.width.unwrap_or(800) as f64, height: options.height.unwrap_or(600) as f64,
            },
            renderer: crate::types::WebviewRendererKind::Native,
        };
        match crate::webview::create_webview(opts) {
            Ok(id) => Ok(id),
            Err(e) => Err(WebViewError { message: e.to_string() }),
        }
    }

    fn close_webview(&self, webview_id: u32) -> Result<(), WebViewError> {
        if crate::webview::close_webview(webview_id) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn navigate(&self, webview_id: u32, url: String) -> Result<(), WebViewError> {
        if crate::webview::webview_navigate(webview_id, &url) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn navigate_back(&self, webview_id: u32) -> Result<(), WebViewError> {
        if crate::webview::webview_go_back(webview_id) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn navigate_forward(&self, webview_id: u32) -> Result<(), WebViewError> {
        if crate::webview::webview_go_forward(webview_id) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn reload(&self, webview_id: u32) -> Result<(), WebViewError> {
        if crate::webview::webview_reload(webview_id) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn load_html(&self, webview_id: u32, html: String) -> Result<(), WebViewError> {
        if crate::webview::load_html(webview_id, &html) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn set_webview_bounds(&self, webview_id: u32, bounds: WebViewBounds) -> Result<(), WebViewError> {
        if crate::webview::set_webview_bounds(webview_id, bounds.x, bounds.y, bounds.width, bounds.height) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn set_webview_visible(&self, webview_id: u32, visible: bool) -> Result<(), WebViewError> {
        if crate::webview::set_webview_visible(webview_id, visible) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }

    fn resize(&self, webview_id: u32, width: f64, height: f64) -> Result<(), WebViewError> {
        match crate::webview::get_webview_bounds(webview_id) {
            Some(b) => {
                if crate::webview::set_webview_bounds(webview_id, b.x, b.y, width, height) { Ok(()) }
                else { Err(WebViewError { message: format!("WebView {} resize failed", webview_id) }) }
            }
            None => Err(WebViewError { message: format!("WebView {} not found", webview_id) }),
        }
    }

    fn send_message(&self, webview_id: u32, message: String) -> Result<(), WebViewError> {
        if crate::webview::send_message_to_webview(webview_id, &message) { Ok(()) }
        else { Err(WebViewError { message: format!("WebView {} not found", webview_id) }) }
    }
}

// ============================================================================
// Dialog Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub filters: Option<Vec<String>>,
    pub multi_selections: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogResult {
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBoxKind { Info, Warning, Error, Question, }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBoxResult { pub clicked: String, }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogError { pub message: String, }

// ============================================================================
// Tray Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayOptions {
    pub image: String,
    pub title: Option<String>,
    pub menu_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayBounds { pub x: f64, pub y: f64, pub width: f64, pub height: f64, }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayError { pub message: String, }

// ============================================================================
// Clipboard Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardError { pub message: String, }

// ============================================================================
// Dialog API
// ============================================================================

#[eden_ipc]
pub trait DialogAPI {
    fn open_file_dialog(&self, options: FileDialogOptions) -> Result<FileDialogResult, DialogError>;
    fn show_message_box(&self, title: String, message: String, kind: String) -> Result<MessageBoxResult, DialogError>;
    fn move_to_trash(&self, path: String) -> Result<(), DialogError>;
}

impl DialogAPI for ElectrobunApp {
    fn open_file_dialog(&self, options: FileDialogOptions) -> Result<FileDialogResult, DialogError> {
        let filters: Vec<(String, Vec<String>)> = Vec::new();
        match crate::dialog::open_file_dialog(
            &options.title.unwrap_or_default(),
            &options.default_path.unwrap_or_default(),
            &filters, options.multi_selections,
        ) {
            Ok(files) => Ok(FileDialogResult { files }),
            Err(e) => Err(DialogError { message: e.to_string() }),
        }
    }

    fn show_message_box(&self, title: String, message: String, kind: String) -> Result<MessageBoxResult, DialogError> {
        use crate::dialog::MessageBoxKind;
        let mb_kind = match kind.as_str() {
            "warning" => MessageBoxKind::Warning, "error" => MessageBoxKind::Error,
            "question" => MessageBoxKind::Question, _ => MessageBoxKind::Info,
        };
        let result = crate::dialog::show_message_box(&title, &message, mb_kind);
        Ok(MessageBoxResult { clicked: format!("{:?}", result) })
    }

    fn move_to_trash(&self, path: String) -> Result<(), DialogError> {
        match crate::dialog::move_to_trash(&path) {
            Ok(()) => Ok(()),
            Err(e) => Err(DialogError { message: e.to_string() }),
        }
    }
}

// ============================================================================
// Tray API
// ============================================================================

#[eden_ipc]
pub trait TrayAPI {
    fn create_tray(&self, options: TrayOptions) -> Result<u32, TrayError>;
    fn destroy_tray(&self, tray_id: u32) -> Result<(), TrayError>;
    fn set_tray_image(&self, tray_id: u32, image: String) -> Result<(), TrayError>;
    fn set_tray_title(&self, tray_id: u32, title: String) -> Result<(), TrayError>;
    fn show_tray(&self, tray_id: u32) -> Result<(), TrayError>;
    fn hide_tray(&self, tray_id: u32) -> Result<(), TrayError>;
    fn set_tray_menu(&self, tray_id: u32, menu_json: String) -> Result<(), TrayError>;
}

impl TrayAPI for ElectrobunApp {
    fn create_tray(&self, options: TrayOptions) -> Result<u32, TrayError> {
        match crate::tray::create_tray(&options.image, &options.title.unwrap_or_default()) {
            Ok(id) => Ok(id),
            Err(e) => Err(TrayError { message: e.to_string() }),
        }
    }
    fn destroy_tray(&self, tray_id: u32) -> Result<(), TrayError> {
        if crate::tray::destroy_tray(tray_id) { Ok(()) }
        else { Err(TrayError { message: format!("Tray {} not found", tray_id) }) }
    }
    fn set_tray_image(&self, tray_id: u32, image: String) -> Result<(), TrayError> {
        if crate::tray::set_tray_image(tray_id, &image) { Ok(()) }
        else { Err(TrayError { message: format!("Tray {} not found", tray_id) }) }
    }
    fn set_tray_title(&self, tray_id: u32, title: String) -> Result<(), TrayError> {
        if crate::tray::set_tray_title(tray_id, &title) { Ok(()) }
        else { Err(TrayError { message: format!("Tray {} not found", tray_id) }) }
    }
    fn show_tray(&self, tray_id: u32) -> Result<(), TrayError> {
        if crate::tray::show_tray(tray_id) { Ok(()) }
        else { Err(TrayError { message: format!("Tray {} not found", tray_id) }) }
    }
    fn hide_tray(&self, tray_id: u32) -> Result<(), TrayError> {
        if crate::tray::hide_tray(tray_id) { Ok(()) }
        else { Err(TrayError { message: format!("Tray {} not found", tray_id) }) }
    }
    fn set_tray_menu(&self, tray_id: u32, menu_json: String) -> Result<(), TrayError> {
        if crate::tray::set_tray_menu(tray_id, &menu_json) { Ok(()) }
        else { Err(TrayError { message: format!("Tray {} not found", tray_id) }) }
    }
}

// ============================================================================
// Clipboard API
// ============================================================================

#[eden_ipc]
pub trait ClipboardAPI {
    fn read_text(&self) -> Result<String, ClipboardError>;
    fn write_text(&self, text: String) -> Result<(), ClipboardError>;
    fn clear(&self) -> Result<(), ClipboardError>;
}

impl ClipboardAPI for ElectrobunApp {
    fn read_text(&self) -> Result<String, ClipboardError> {
        match crate::clipboard::read_text() {
            Ok(text) => Ok(text),
            Err(e) => Err(ClipboardError { message: e.to_string() }),
        }
    }
    fn write_text(&self, text: String) -> Result<(), ClipboardError> {
        match crate::clipboard::write_text(&text) {
            Ok(()) => Ok(()),
            Err(e) => Err(ClipboardError { message: e.to_string() }),
        }
    }
    fn clear(&self) -> Result<(), ClipboardError> {
        match crate::clipboard::clear() {
            Ok(()) => Ok(()),
            Err(e) => Err(ClipboardError { message: e.to_string() }),
        }
    }
}

// ============================================================================
// Session API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionError { pub message: String, }

#[eden_ipc]
pub trait SessionAPI {
    fn get_cookies(&self, url: String) -> Result<String, SessionError>;
    fn set_cookie(&self, url: String, name: String, value: String, domain: Option<String>, path: Option<String>, secure: bool, http_only: bool, max_age: Option<i64>) -> Result<(), SessionError>;
    fn remove_cookie(&self, url: String, name: String) -> Result<(), SessionError>;
    fn clear_cookies(&self) -> Result<(), SessionError>;
    fn clear_storage_data(&self) -> Result<(), SessionError>;
}

impl SessionAPI for ElectrobunApp {
    fn get_cookies(&self, url: String) -> Result<String, SessionError> {
        crate::session::get_cookies(&url).map_err(|e| SessionError { message: e.to_string() })
    }
    fn set_cookie(&self, url: String, name: String, value: String, domain: Option<String>, path: Option<String>, secure: bool, http_only: bool, max_age: Option<i64>) -> Result<(), SessionError> {
        crate::session::set_cookie(&url, &name, &value, domain.as_deref(), path.as_deref(), secure, http_only, max_age).map_err(|e| SessionError { message: e.to_string() })
    }
    fn remove_cookie(&self, url: String, name: String) -> Result<(), SessionError> {
        crate::session::remove_cookie(&url, &name).map_err(|e| SessionError { message: e.to_string() })
    }
    fn clear_cookies(&self) -> Result<(), SessionError> {
        crate::session::clear_cookies().map_err(|e| SessionError { message: e.to_string() })
    }
    fn clear_storage_data(&self) -> Result<(), SessionError> {
        crate::session::clear_storage_data().map_err(|e| SessionError { message: e.to_string() })
    }
}

// ============================================================================
// Shortcuts API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutError { pub message: String, }

#[eden_ipc]
pub trait ShortcutsAPI {
    fn register(&self, accelerator: String) -> bool;
    fn unregister(&self, accelerator: String) -> bool;
    fn unregister_all(&self) -> bool;
    fn is_registered(&self, accelerator: String) -> bool;
}

impl ShortcutsAPI for ElectrobunApp {
    fn register(&self, accelerator: String) -> bool { crate::shortcuts::register_shortcut(&accelerator) }
    fn unregister(&self, accelerator: String) -> bool { crate::shortcuts::unregister_shortcut(&accelerator) }
    fn unregister_all(&self) -> bool { crate::shortcuts::unregister_all() }
    fn is_registered(&self, accelerator: String) -> bool { crate::shortcuts::is_shortcut_registered(&accelerator) }
}

// ============================================================================
// Display API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: i64,
    pub x: f64, pub y: f64, pub width: f64, pub height: f64,
    pub work_x: f64, pub work_y: f64, pub work_width: f64, pub work_height: f64,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayError { pub message: String, }

#[eden_ipc]
pub trait DisplayAPI {
    fn get_primary_display(&self) -> Result<DisplayInfo, DisplayError>;
    fn get_all_displays(&self) -> Result<Vec<DisplayInfo>, DisplayError>;
    fn get_cursor_screen_point(&self) -> Result<(f64, f64), DisplayError>;
}

impl DisplayAPI for ElectrobunApp {
    fn get_primary_display(&self) -> Result<DisplayInfo, DisplayError> {
        crate::display::get_primary_display().map_err(|e| DisplayError { message: e.to_string() }).map(|d| DisplayInfo {
            id: d.id, x: d.bounds.x, y: d.bounds.y, width: d.bounds.width, height: d.bounds.height,
            work_x: d.work_area.x, work_y: d.work_area.y, work_width: d.work_area.width, work_height: d.work_area.height,
            scale_factor: d.scale_factor, is_primary: d.is_primary, name: d.name,
        })
    }
    fn get_all_displays(&self) -> Result<Vec<DisplayInfo>, DisplayError> {
        crate::display::get_all_displays().map_err(|e| DisplayError { message: e.to_string() }).map(|displays| displays.into_iter().map(|d| DisplayInfo {
            id: d.id, x: d.bounds.x, y: d.bounds.y, width: d.bounds.width, height: d.bounds.height,
            work_x: d.work_area.x, work_y: d.work_area.y, work_width: d.work_area.width, work_height: d.work_area.height,
            scale_factor: d.scale_factor, is_primary: d.is_primary, name: d.name,
        }).collect())
    }
    fn get_cursor_screen_point(&self) -> Result<(f64, f64), DisplayError> {
        crate::display::get_cursor_screen_point().map_err(|e| DisplayError { message: e.to_string() })
    }
}

// ============================================================================
// Core (Lifecycle) API
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreError { pub message: String, }

#[eden_ipc]
pub trait CoreAPI {
    fn quit_gracefully(&self);
    fn stop_event_loop(&self);
    fn force_exit(&self, code: i32);
    fn set_exit_on_last_window_closed(&self, value: bool);
    fn get_platform(&self) -> String;
}

impl CoreAPI for ElectrobunApp {
    fn quit_gracefully(&self) { crate::window::shutdown(); }
    fn stop_event_loop(&self) { crate::window::shutdown(); }
    fn force_exit(&self, code: i32) { std::process::exit(code); }
    fn set_exit_on_last_window_closed(&self, value: bool) { unsafe { crate::EXIT_ON_LAST_WINDOW_CLOSED = value; } }
    fn get_platform(&self) -> String {
        if cfg!(target_os = "windows") { "windows".to_string() }
        else if cfg!(target_os = "macos") { "macos".to_string() }
        else { "linux".to_string() }
    }
}

// ============================================================================
// WGPU API (handle-based, no raw pointers across FFI)
// ============================================================================

use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgpuAdapterOpts {
    pub power_preference: Option<String>,
    pub force_fallback_adapter: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgpuDeviceOpts {
    pub required_features: Option<Vec<String>>,
    pub required_limits: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgpuSurfaceOpts {
    pub width: u32,
    pub height: u32,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WgpuError {
    pub message: String,
}

/// 全局 WGPU 资源管理器（仅 Rust 内部使用，TS 只传 u64 ID）
enum WgpuResource {
    Adapter(wgpu::Adapter),
    Device(wgpu::Device),
    Surface(wgpu::Surface<'static>),
    ShaderModule(wgpu::ShaderModule),
    Pipeline(wgpu::RenderPipeline),
    Buffer(wgpu::Buffer),
}

struct WgpuHub {
    next_id: u64,
    resources: std::collections::HashMap<u64, WgpuResource>,
}

impl WgpuHub {
    fn new() -> Self {
        Self { next_id: 1, resources: std::collections::HashMap::new() }
    }
    fn register(&mut self, resource: WgpuResource) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.resources.insert(id, resource);
        id
    }
    fn get(&self, id: u64) -> Option<&WgpuResource> {
        self.resources.get(&id)
    }
    fn remove(&mut self, id: u64) -> Option<WgpuResource> {
        self.resources.remove(&id)
    }
}

static WGPU_HUB: LazyLock<std::sync::Mutex<WgpuHub>> =
    LazyLock::new(|| std::sync::Mutex::new(WgpuHub::new()));

mod wgpu_hub {
    use super::*;

    pub fn request_adapter(opts: WgpuAdapterOpts) -> Result<u64, WgpuError> {
        let instance = wgpu::Instance::default();
        let power_pref = match opts.power_preference.as_deref() {
            Some("high") => wgpu::PowerPreference::HighPerformance,
            Some("low") => wgpu::PowerPreference::LowPower,
            _ => wgpu::PowerPreference::HighPerformance,
        };
        // wgpu 29: pollster not available, use simple blocking
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power_pref,
            force_fallback_adapter: opts.force_fallback_adapter.unwrap_or(false),
            compatible_surface: None,
        })).map_err(|_| WgpuError { message: "No adapter found".into() })?;
        let mut hub = WGPU_HUB.lock().unwrap();
        Ok(hub.register(WgpuResource::Adapter(adapter)))
    }

    pub fn request_device(adapter_id: u64, _opts: WgpuDeviceOpts) -> Result<u64, WgpuError> {
        Err(WgpuError { message: "device request not yet implemented".into() })
    }

    pub fn create_surface(_device_id: u64, _opts: WgpuSurfaceOpts) -> Result<u64, WgpuError> {
        Err(WgpuError { message: "Surface creation requires a raw window handle; not yet implemented".into() })
    }

    pub fn present(_surface_id: u64) -> Result<(), WgpuError> {
        Ok(())
    }

    pub fn destroy(handle: u64) -> Result<(), WgpuError> {
        let mut hub = WGPU_HUB.lock().unwrap();
        hub.remove(handle);
        Ok(())
    }
}

#[eden_ipc]
pub trait WgpuAPI {
    fn request_adapter(&self, opts: WgpuAdapterOpts) -> Result<u64, WgpuError>;
    fn request_device(&self, adapter_id: u64, opts: WgpuDeviceOpts) -> Result<u64, WgpuError>;
    fn create_surface(&self, device_id: u64, opts: WgpuSurfaceOpts) -> Result<u64, WgpuError>;
    fn present(&self, surface_id: u64) -> Result<(), WgpuError>;
    fn destroy(&self, handle: u64) -> Result<(), WgpuError>;
}

impl WgpuAPI for ElectrobunApp {
    fn request_adapter(&self, opts: WgpuAdapterOpts) -> Result<u64, WgpuError> {
        wgpu_hub::request_adapter(opts)
    }
    fn request_device(&self, adapter_id: u64, opts: WgpuDeviceOpts) -> Result<u64, WgpuError> {
        wgpu_hub::request_device(adapter_id, opts)
    }
    fn create_surface(&self, device_id: u64, opts: WgpuSurfaceOpts) -> Result<u64, WgpuError> {
        wgpu_hub::create_surface(device_id, opts)
    }
    fn present(&self, surface_id: u64) -> Result<(), WgpuError> {
        wgpu_hub::present(surface_id)
    }
    fn destroy(&self, handle: u64) -> Result<(), WgpuError> {
        wgpu_hub::destroy(handle)
    }
}
