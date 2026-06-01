//! Electrobun Core - Rust Port
//! Cross-platform desktop application framework core library
//!
//! This crate provides the FFI interface for the Electrobun framework,
//! which uses Bun as the JavaScript runtime (similar to Electron but with Bun).

#![allow(non_upper_case_globals)]
#![allow(unused)]

pub mod error;
pub mod types;
pub mod window;
pub mod webview;
pub mod transport;
pub mod crypto;
pub mod tray;
pub mod wgpu;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

// Re-export commonly used types
pub use error::ElectrobunError;
pub use types::*;

// Global state using thread-safe primitives
// Zig: var window_registry = std.AutoHashMap(u32, WindowState).init(allocator)
lazy_static::lazy_static! {
    static ref WINDOW_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, window::WindowState>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    static ref WEBVIEW_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, webview::WebviewState>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    static ref WGPU_VIEW_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, wgpu::WgpuViewState>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    static ref TRAY_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, tray::TrayState>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    
    static ref NEXT_WINDOW_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    static ref NEXT_WEBVIEW_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    static ref NEXT_WGPU_VIEW_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    static ref NEXT_TRAY_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    
    static ref LAST_ERROR: Arc<Mutex<Option<CString>>> = Arc::new(Mutex::new(None));
}

/// Get the next available window ID
fn next_window_id() -> u32 {
    let mut id = NEXT_WINDOW_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

/// Get the next available webview ID
fn next_webview_id() -> u32 {
    let mut id = NEXT_WEBVIEW_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

/// Get the next available wgpu view ID
fn next_wgpu_view_id() -> u32 {
    let mut id = NEXT_WGPU_VIEW_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

/// Get the next available tray ID
fn next_tray_id() -> u32 {
    let mut id = NEXT_TRAY_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

/// Clear the last error
fn clear_last_error() {
    let mut err = LAST_ERROR.lock().unwrap();
    *err = None;
}

/// Set the last error
fn set_last_error(msg: &str) {
    let mut err = LAST_ERROR.lock().unwrap();
    *err = Some(CString::new(msg).unwrap_or_else(|_| CString::new("Unknown error").unwrap()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// FFI EXPORTS - These functions are exported for Bun/TypeScript to call
// ═══════════════════════════════════════════════════════════════════════════════

/// Get the last error message (FFI)
/// Zig: export fn electrobun_core_last_error() [*:0]const u8
#[no_mangle]
pub extern "C" fn electrobun_core_last_error() -> *const c_char {
    let err = LAST_ERROR.lock().unwrap();
    match err.as_ref() {
        Some(cstr) => cstr.as_ptr(),
        None => ptr::null(),
    }
}

/// Get the wakeup file descriptor for message polling (FFI)
/// Zig: export fn getHostMessageWakeupReadFD() c_int
#[no_mangle]
pub extern "C" fn electrobun_get_host_message_wakeup_read_fd() -> std::os::raw::c_int {
    transport::get_wakeup_fd().unwrap_or(-1)
}

/// Free a core-allocated string (FFI)
/// Zig: export fn freeCoreString(value: ?[*:0]u8) void
#[no_mangle]
pub extern "C" fn electrobun_free_core_string(value: *mut c_char) {
    if !value.is_null() {
        // String was allocated by us, need to deallocate
        // In Rust, we use Box::from_raw to take ownership
        unsafe {
            let _ = CString::from_raw(value);
        }
    }
}

/// Initialize the webview runtime (FFI)
/// Zig: export fn initWebviewRuntime(...) void
#[no_mangle]
pub extern "C" fn electrobun_init_webview_runtime(
    preload_script: *const c_char,
    preload_script_sandboxed: *const c_char,
) -> bool {
    clear_last_error();
    
    let preload = if preload_script.is_null() {
        None
    } else {
        unsafe { Some(CStr::from_ptr(preload_script).to_string_lossy().into_owned()) }
    };
    
    let preload_sandboxed = if preload_script_sandboxed.is_null() {
        None
    } else {
        unsafe { Some(CStr::from_ptr(preload_script_sandboxed).to_string_lossy().into_owned()) }
    };
    
    // Store in global state
    let mut state = webview::WEBVIEW_RUNTIME_STATE.lock().unwrap();
    state.preload_script = preload;
    state.preload_script_sandboxed = preload_sandboxed;
    state.configured = true;
    
    true
}

// ═══════════════════════════════════════════════════════════════════════════════
// WINDOW FUNCTIONS - FFI exports
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a new window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_create_window(
    title: *const c_char,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    options_bits: u32, // bitfield for various options
) -> u32 {
    clear_last_error();
    
    let title_str = if title.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(title).to_string_lossy().into_owned() }
    };
    
    let options = window::WindowOptions {
        title: title_str,
        x,
        y,
        width,
        height,
        transparent: (options_bits & 0x01) != 0,
        hidden: (options_bits & 0x02) != 0,
        decorate: (options_bits & 0x04) == 0,
        resizable: (options_bits & 0x08) != 0,
        closable: (options_bits & 0x10) != 0,
        movable: (options_bits & 0x20) != 0,
        minimizable: (options_bits & 0x40) != 0,
        maximizable: (options_bits & 0x80) != 0,
        focusable: (options_bits & 0x100) != 0,
        always_on_top: (options_bits & 0x200) != 0,
        always_on_bottom: (options_bits & 0x400) != 0,
        fullscreen: (options_bits & 0x800) != 0,
    };
    
    match window::create_window(options) {
        Ok(id) => id,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Close a window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_close_window(id: u32) -> bool {
    clear_last_error();
    window::close_window(id)
}

/// Show a window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_show_window(id: u32) -> bool {
    clear_last_error();
    window::show_window(id)
}

/// Hide a window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_hide_window(id: u32) -> bool {
    clear_last_error();
    window::hide_window(id)
}

/// Minimize a window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_minimize_window(id: u32) -> bool {
    clear_last_error();
    window::minimize_window(id)
}

/// Maximize a window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_maximize_window(id: u32) -> bool {
    clear_last_error();
    window::maximize_window(id)
}

/// Unmaximize a window (FFI)
#[no_mangle]
pub extern "C" fn electrobun_unmaximize_window(id: u32) -> bool {
    clear_last_error();
    window::unmaximize_window(id)
}

/// Set window title (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_window_title(id: u32, title: *const c_char) -> bool {
    clear_last_error();
    let title_str = if title.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(title).to_string_lossy().into_owned() }
    };
    window::set_window_title(id, &title_str)
}

/// Set window bounds (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_window_bounds(
    id: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> bool {
    clear_last_error();
    window::set_window_bounds(id, x, y, width, height)
}

/// Get window bounds (FFI)
#[no_mangle]
pub extern "C" fn electrobun_get_window_bounds(
    id: u32,
    out_x: *mut f64,
    out_y: *mut f64,
    out_width: *mut f64,
    out_height: *mut f64,
) -> bool {
    clear_last_error();
    
    if out_x.is_null() || out_y.is_null() || out_width.is_null() || out_height.is_null() {
        return false;
    }
    
    match window::get_window_bounds(id) {
        Some(bounds) => {
            unsafe {
                *out_x = bounds.x;
                *out_y = bounds.y;
                *out_width = bounds.width;
                *out_height = bounds.height;
            }
            true
        }
        None => false,
    }
}

/// Set window always on top (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_window_always_on_top(id: u32, on_top: bool) -> bool {
    clear_last_error();
    window::set_window_always_on_top(id, on_top)
}

/// Set window focus (FFI)
#[no_mangle]
pub extern "C" fn electrobun_focus_window(id: u32) -> bool {
    clear_last_error();
    window::focus_window(id)
}

/// Set window fullscreen (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_window_fullscreen(id: u32, fullscreen: bool) -> bool {
    clear_last_error();
    window::set_window_fullscreen(id, fullscreen)
}

/// Set window frame (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_window_frame(id: u32, frameless: bool) -> bool {
    clear_last_error();
    window::set_window_frame(id, frameless)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBVIEW FUNCTIONS - FFI exports
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a webview (FFI)
#[no_mangle]
pub extern "C" fn electrobun_create_webview(
    window_id: u32,
    url: *const c_char,
    secret_key: *const c_char,
    partition: *const c_char,
    sandboxed: bool,
) -> u32 {
    clear_last_error();
    
    let url_str = if url.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() }
    };
    
    let secret_key_str = if secret_key.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(secret_key).to_string_lossy().into_owned() }
    };
    
    let partition_str = if partition.is_null() {
        String::from("persist:default")
    } else {
        unsafe { CStr::from_ptr(partition).to_string_lossy().into_owned() }
    };
    
    let options = webview::WebviewOptions {
        window_id,
        url: url_str,
        secret_key: secret_key_str,
        partition: partition_str,
        sandboxed,
        ..Default::default()
    };
    
    match webview::create_webview(options) {
        Ok(id) => id,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Close a webview (FFI)
#[no_mangle]
pub extern "C" fn electrobun_close_webview(id: u32) -> bool {
    clear_last_error();
    webview::close_webview(id)
}

/// Navigate webview to URL (FFI)
#[no_mangle]
pub extern "C" fn electrobun_webview_navigate(id: u32, url: *const c_char) -> bool {
    clear_last_error();
    let url_str = if url.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(url).to_string_lossy().into_owned() }
    };
    webview::webview_navigate(id, &url_str)
}

/// Set webview bounds (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_webview_bounds(
    id: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> bool {
    clear_last_error();
    webview::set_webview_bounds(id, x, y, width, height)
}

/// Get webview bounds (FFI)
#[no_mangle]
pub extern "C" fn electrobun_get_webview_bounds(
    id: u32,
    out_x: *mut f64,
    out_y: *mut f64,
    out_width: *mut f64,
    out_height: *mut f64,
) -> bool {
    clear_last_error();
    
    if out_x.is_null() || out_y.is_null() || out_width.is_null() || out_height.is_null() {
        return false;
    }
    
    match webview::get_webview_bounds(id) {
        Some(bounds) => {
            unsafe {
                *out_x = bounds.x;
                *out_y = bounds.y;
                *out_width = bounds.width;
                *out_height = bounds.height;
            }
            true
        }
        None => false,
    }
}

/// Show/hide webview (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_webview_visible(id: u32, visible: bool) -> bool {
    clear_last_error();
    webview::set_webview_visible(id, visible)
}

/// Set webview transparent (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_webview_transparent(id: u32, transparent: bool) -> bool {
    clear_last_error();
    webview::set_webview_transparent(id, transparent)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAY FUNCTIONS - FFI exports
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a tray (FFI)
#[no_mangle]
pub extern "C" fn electrobun_create_tray(
    image: *const c_char,
    title: *const c_char,
) -> u32 {
    clear_last_error();
    
    let image_str = if image.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(image).to_string_lossy().into_owned() }
    };
    
    let title_str = if title.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(title).to_string_lossy().into_owned() }
    };
    
    match tray::create_tray(&image_str, &title_str) {
        Ok(id) => id,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Destroy a tray (FFI)
#[no_mangle]
pub extern "C" fn electrobun_destroy_tray(id: u32) -> bool {
    clear_last_error();
    tray::destroy_tray(id)
}

/// Set tray image (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_tray_image(id: u32, image: *const c_char) -> bool {
    clear_last_error();
    let image_str = if image.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(image).to_string_lossy().into_owned() }
    };
    tray::set_tray_image(id, &image_str)
}

/// Set tray title (FFI)
#[no_mangle]
pub extern "C" fn electrobun_set_tray_title(id: u32, title: *const c_char) -> bool {
    clear_last_error();
    let title_str = if title.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(title).to_string_lossy().into_owned() }
    };
    tray::set_tray_title(id, &title_str)
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOST MESSAGE FUNCTIONS - FFI exports
// ═══════════════════════════════════════════════════════════════════════════════

/// Pop the next queued host message (FFI)
/// Zig: export fn popNextQueuedHostMessage(out_webview_id: *u32) ?[*:0]u8
#[no_mangle]
pub extern "C" fn electrobun_pop_next_queued_host_message(
    out_webview_id: *mut u32,
) -> *mut c_char {
    clear_last_error();
    
    if out_webview_id.is_null() {
        return ptr::null_mut();
    }
    
    match transport::pop_next_message() {
        Some((webview_id, message)) => {
            unsafe { *out_webview_id = webview_id; }
            // Allocate a CString to return
            CString::new(message)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        }
        None => {
            unsafe { *out_webview_id = 0; }
            ptr::null_mut()
        }
    }
}

/// Send message to webview (FFI)
#[no_mangle]
pub extern "C" fn electrobun_send_message_to_webview(
    webview_id: u32,
    message: *const c_char,
) -> bool {
    clear_last_error();
    
    let message_str = if message.is_null() {
        return false;
    } else {
        unsafe { CStr::from_ptr(message).to_string_lossy().into_owned() }
    };
    
    transport::send_message_to_webview(webview_id, &message_str)
}

/// Register default webview callbacks (FFI)
#[no_mangle]
pub extern "C" fn electrobun_register_webview_callbacks(
    navigation_callback: Option<unsafe extern "C" fn(u32, *const c_char) -> u32>,
    event_callback: Option<unsafe extern "C" fn(u32, *const c_char, *const c_char)>,
    bridge_callback: Option<unsafe extern "C" fn(u32, *const c_char)>,
) {
    let mut callbacks = webview::DEFAULT_WEBVIEW_CALLBACKS.lock().unwrap();
    callbacks.navigation_callback = navigation_callback.map(|f| Arc::new(f));
    callbacks.event_callback = event_callback.map(|f| Arc::new(f));
    callbacks.bridge_callback = bridge_callback.map(|f| Arc::new(f));
}

// ═══════════════════════════════════════════════════════════════════════════════
// PLATFORM INFO
// ═══════════════════════════════════════════════════════════════════════════════

/// Get the current platform name
#[no_mangle]
pub extern "C" fn electrobun_get_platform() -> *const c_char {
    if cfg!(target_os = "macos") {
        c"macos".as_ptr() as *const c_char
    } else if cfg!(target_os = "windows") {
        c"windows".as_ptr() as *const c_char
    } else if cfg!(target_os = "linux") {
        c"linux".as_ptr() as *const c_char
    } else {
        c"unknown".as_ptr() as *const c_char
    }
}
