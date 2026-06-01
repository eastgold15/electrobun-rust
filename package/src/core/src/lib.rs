//! Electrobun Core — Rust FFI Library
//!
//! Provides the FFI interface for the Electrobun framework.
//! All functions are #[no_mangle] pub extern "C" for C ABI compatibility with Bun/TypeScript.

#![allow(non_upper_case_globals)]
#![allow(unused)]

pub mod clipboard;
pub mod dialog;
pub mod display;
pub mod error;
pub mod file_ops;
pub mod notifications;
pub mod session;
pub mod shortcuts;
pub mod transport;
pub mod tray;
pub mod types;
pub mod webview;
pub mod wgpu;
pub mod window;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

pub use error::ElectrobunError;
pub use types::*;

lazy_static::lazy_static! {
    static ref WINDOW_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, types::WindowState>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    static ref WEBVIEW_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, types::WebviewState>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    static ref WGPU_VIEW_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, types::WgpuViewState>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    static ref TRAY_REGISTRY: Arc<Mutex<std::collections::HashMap<u32, types::TrayState>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    static ref NEXT_WINDOW_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    static ref NEXT_WEBVIEW_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    static ref NEXT_WGPU_VIEW_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));
    static ref NEXT_TRAY_ID: Arc<Mutex<u32>> = Arc::new(Mutex::new(1));

    static ref LAST_ERROR: Arc<Mutex<Option<CString>>> = Arc::new(Mutex::new(None));
}

fn next_window_id() -> u32 {
    let mut id = NEXT_WINDOW_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

fn next_webview_id() -> u32 {
    let mut id = NEXT_WEBVIEW_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

fn next_wgpu_view_id() -> u32 {
    let mut id = NEXT_WGPU_VIEW_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

fn next_tray_id() -> u32 {
    let mut id = NEXT_TRAY_ID.lock().unwrap();
    let result = *id;
    *id += 1;
    result
}

fn clear_last_error() {
    let mut err = LAST_ERROR.lock().unwrap();
    *err = None;
}

fn set_last_error(msg: &str) {
    let mut err = LAST_ERROR.lock().unwrap();
    *err = Some(CString::new(msg).unwrap_or_else(|_| CString::new("Unknown error").unwrap()));
}

fn str_from_ptr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn opt_str_from_ptr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ERROR / MEMORY (4)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_core_last_error() -> *const c_char {
    let err = LAST_ERROR.lock().unwrap();
    match err.as_ref() {
        Some(cstr) => cstr.as_ptr(),
        None => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn electrobun_free_core_string(value: *mut c_char) {
    if !value.is_null() {
        unsafe { let _ = CString::from_raw(value); }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_pop_next_queued_host_message(out_webview_id: *mut u32) -> *mut c_char {
    clear_last_error();
    if out_webview_id.is_null() {
        return ptr::null_mut();
    }
    match webview::pop_next_message() {
        Some((webview_id, message)) => {
            unsafe { *out_webview_id = webview_id; }
            CString::new(message).map(|s| s.into_raw()).unwrap_or(ptr::null_mut())
        }
        None => {
            unsafe { *out_webview_id = 0; }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_host_message_wakeup_read_fd() -> std::os::raw::c_int {
    clear_last_error();
    transport::get_wakeup_fd().unwrap_or(-1)
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIFECYCLE (6)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_configure_webview_runtime(
    rpc_port: u32,
    preload_script: *const c_char,
    preload_script_sandboxed: *const c_char,
) -> bool {
    clear_last_error();
    let preload = opt_str_from_ptr(preload_script);
    let preload_sandboxed = opt_str_from_ptr(preload_script_sandboxed);
    let mut state = webview::WEBVIEW_RUNTIME_STATE.lock().unwrap();
    state.rpc_port = rpc_port;
    state.preload_script = preload;
    state.preload_script_sandboxed = preload_sandboxed;
    state.configured = true;
    true
}

#[no_mangle]
pub extern "C" fn electrobun_init_webview_runtime(
    preload_script: *const c_char,
    preload_script_sandboxed: *const c_char,
) -> bool {
    clear_last_error();
    let preload = opt_str_from_ptr(preload_script);
    let preload_sandboxed = opt_str_from_ptr(preload_script_sandboxed);
    let mut state = webview::WEBVIEW_RUNTIME_STATE.lock().unwrap();
    state.preload_script = preload;
    state.preload_script_sandboxed = preload_sandboxed;
    state.configured = true;
    true
}

#[no_mangle]
pub extern "C" fn electrobun_core_run_main_thread() -> bool {
    clear_last_error();
    // The winit event loop is already running in a background thread
    // This function ensures the event loop is initialized
    window::init_event_loop();
    true
}

/// Set URL open handler (for macOS open-url events)
#[no_mangle]
pub extern "C" fn electrobun_set_url_open_handler(_handler: Option<unsafe extern "C" fn(*const c_char)>) {
    // TODO: Store and invoke URL open handler
}

/// Set app reopen handler (for macOS reopen events)
#[no_mangle]
pub extern "C" fn electrobun_set_app_reopen_handler(_handler: Option<unsafe extern "C" fn()>) {
    // TODO: Store and invoke app reopen handler
}

/// Set quit requested handler
#[no_mangle]
pub extern "C" fn electrobun_set_quit_requested_handler(_handler: Option<unsafe extern "C" fn()>) {
    // TODO: Store and invoke quit requested handler
}

/// Set exit on last window closed
pub static mut EXIT_ON_LAST_WINDOW_CLOSED: bool = true;

#[no_mangle]
pub extern "C" fn electrobun_set_exit_on_last_window_closed(value: bool) {
    unsafe { EXIT_ON_LAST_WINDOW_CLOSED = value; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT LOOP CONTROL (4)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_quit_gracefully() {
    window::shutdown();
}

#[no_mangle]
pub extern "C" fn electrobun_stop_event_loop() {
    window::shutdown();
}

#[no_mangle]
pub extern "C" fn electrobun_wait_for_shutdown_complete() {
    // The join handle in window::shutdown already waits
}

#[no_mangle]
pub extern "C" fn electrobun_force_exit(code: std::os::raw::c_int) {
    std::process::exit(code);
}

// ═══════════════════════════════════════════════════════════════════════════════
// WINDOW MANAGEMENT (27)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_get_window_style(_id: u32) -> *mut c_char {
    clear_last_error();
    // Return default window style as JSON
    let style = r#"{"borderless":false,"titled":true,"closable":true,"miniaturizable":true,"resizable":true}"#;
    CString::new(style).map(|s| s.into_raw()).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn electrobun_create_window(
    title: *const c_char,
    x: f64, y: f64, width: f64, height: f64,
    options_bits: u32,
) -> u32 {
    clear_last_error();
    let title_str = str_from_ptr(title);
    let options = types::WindowOptions {
        title: title_str,
        x, y, width, height,
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
        Err(e) => { set_last_error(&e.to_string()); 0 }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_window_pointer(id: u32) -> *mut std::ffi::c_void {
    clear_last_error();
    // Window pointer can't be directly returned from winit event loop thread
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_title(id: u32, title: *const c_char) -> bool {
    clear_last_error();
    let title_str = str_from_ptr(title);
    window::set_window_title(id, &title_str)
}

#[no_mangle]
pub extern "C" fn electrobun_minimize_window(id: u32) -> bool {
    clear_last_error();
    window::minimize_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_restore_window(id: u32) -> bool {
    clear_last_error();
    window::restore_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_is_window_minimized(id: u32) -> bool {
    clear_last_error();
    window::is_window_minimized(id).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn electrobun_maximize_window(id: u32) -> bool {
    clear_last_error();
    window::maximize_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_unmaximize_window(id: u32) -> bool {
    clear_last_error();
    window::unmaximize_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_is_window_maximized(id: u32) -> bool {
    clear_last_error();
    window::is_window_maximized(id).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_fullscreen(id: u32, fullscreen: bool) -> bool {
    clear_last_error();
    window::set_window_fullscreen(id, fullscreen)
}

#[no_mangle]
pub extern "C" fn electrobun_is_window_fullscreen(id: u32) -> bool {
    clear_last_error();
    window::is_window_fullscreen(id)
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_always_on_top(id: u32, on_top: bool) -> bool {
    clear_last_error();
    window::set_window_always_on_top(id, on_top)
}

#[no_mangle]
pub extern "C" fn electrobun_is_window_always_on_top(id: u32) -> bool {
    clear_last_error();
    window::is_window_always_on_top(id).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_visible_on_all_workspaces(_id: u32, _visible: bool) -> bool {
    clear_last_error();
    true // Platform-specific; no-op on many platforms
}

#[no_mangle]
pub extern "C" fn electrobun_is_window_visible_on_all_workspaces(_id: u32) -> bool {
    clear_last_error();
    false
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_button_position(_id: u32, _x: f64, _y: f64) -> bool {
    clear_last_error();
    true // macOS-specific (set traffic light position)
}

#[no_mangle]
pub extern "C" fn electrobun_show_window(id: u32) -> bool {
    clear_last_error();
    window::show_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_activate_window(id: u32) -> bool {
    clear_last_error();
    window::focus_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_hide_window(id: u32) -> bool {
    clear_last_error();
    window::hide_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_close_window(id: u32) -> bool {
    clear_last_error();
    window::close_window(id)
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_position(id: u32, x: f64, y: f64) -> bool {
    clear_last_error();
    window::set_window_position(id, x, y)
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_size(id: u32, width: f64, height: f64) -> bool {
    clear_last_error();
    window::set_window_size(id, width, height)
}

#[no_mangle]
pub extern "C" fn electrobun_set_window_frame(id: u32, frameless: bool) -> bool {
    clear_last_error();
    window::set_window_frame(id, frameless)
}

#[no_mangle]
pub extern "C" fn electrobun_get_window_frame(id: u32, out_x: *mut f64, out_y: *mut f64, out_width: *mut f64, out_height: *mut f64) -> bool {
    clear_last_error();
    if out_x.is_null() || out_y.is_null() || out_width.is_null() || out_height.is_null() {
        return false;
    }
    match window::get_window_bounds(id) {
        Some(bounds) => {
            unsafe {
                *out_x = bounds.x; *out_y = bounds.y;
                *out_width = bounds.width; *out_height = bounds.height;
            }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn electrobun_begin_window_move(_id: u32) -> bool {
    clear_last_error();
    true // Platform-specific; needs native window drag
}

#[no_mangle]
pub extern "C" fn electrobun_end_window_move(_id: u32) -> bool {
    clear_last_error();
    true
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBVIEW MANAGEMENT (29)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_create_webview(
    window_id: u32,
    url: *const c_char,
    secret_key: *const c_char,
    partition: *const c_char,
    sandboxed: bool,
) -> u32 {
    clear_last_error();
    let options = types::WebviewOptions {
        window_id,
        url: str_from_ptr(url),
        secret_key: str_from_ptr(secret_key),
        partition: str_from_ptr(partition),
        sandboxed,
        ..Default::default()
    };
    match webview::create_webview(options) {
        Ok(id) => id,
        Err(e) => { set_last_error(&e.to_string()); 0 }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_webview_pointer(_id: u32) -> *mut std::ffi::c_void {
    clear_last_error();
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn electrobun_resize_webview(id: u32, width: f64, height: f64) -> bool {
    clear_last_error();
    let bounds = webview::get_webview_bounds(id);
    if let Some(b) = bounds {
        webview::set_webview_bounds(id, b.x, b.y, width, height)
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn electrobun_load_url_in_webview(id: u32, url: *const c_char) -> bool {
    clear_last_error();
    webview::webview_navigate(id, &str_from_ptr(url))
}

#[no_mangle]
pub extern "C" fn electrobun_load_html_in_webview(id: u32, html: *const c_char) -> bool {
    clear_last_error();
    webview::load_html(id, &str_from_ptr(html))
}

#[no_mangle]
pub extern "C" fn electrobun_update_preload_script_to_webview(id: u32, preload: *const c_char) -> bool {
    clear_last_error();
    webview::update_preload_script(id, &str_from_ptr(preload))
}

#[no_mangle]
pub extern "C" fn electrobun_webview_can_go_back(id: u32) -> bool {
    clear_last_error();
    webview::webview_can_go_back(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_can_go_forward(id: u32) -> bool {
    clear_last_error();
    webview::webview_can_go_forward(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_go_back(id: u32) -> bool {
    clear_last_error();
    webview::webview_go_back(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_go_forward(id: u32) -> bool {
    clear_last_error();
    webview::webview_go_forward(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_reload(id: u32) -> bool {
    clear_last_error();
    webview::webview_reload(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_remove(id: u32) -> bool {
    clear_last_error();
    webview::close_webview(id)
}

#[no_mangle]
pub extern "C" fn electrobun_set_webview_html_content(id: u32, html: *const c_char) -> bool {
    clear_last_error();
    webview::set_webview_html_content(id, &str_from_ptr(html))
}

#[no_mangle]
pub extern "C" fn electrobun_webview_set_transparent(id: u32, transparent: bool) -> bool {
    clear_last_error();
    webview::set_webview_transparent(id, transparent)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_set_passthrough(id: u32, passthrough: bool) -> bool {
    clear_last_error();
    webview::webview_set_passthrough(id, passthrough)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_set_hidden(id: u32, hidden: bool) -> bool {
    clear_last_error();
    webview::webview_set_hidden(id, hidden)
}

#[no_mangle]
pub extern "C" fn electrobun_set_webview_navigation_rules(id: u32, rules_json: *const c_char) -> bool {
    clear_last_error();
    webview::set_navigation_rules(id, &str_from_ptr(rules_json))
}

#[no_mangle]
pub extern "C" fn electrobun_webview_find_in_page(id: u32, search: *const c_char, forward: bool, find_next: bool) -> bool {
    clear_last_error();
    webview::find_in_page(id, &str_from_ptr(search), forward, find_next)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_stop_find(id: u32, clear_selection: bool) -> bool {
    clear_last_error();
    webview::stop_find(id, clear_selection)
}

#[no_mangle]
pub extern "C" fn electrobun_evaluate_javascript_with_no_completion(id: u32, js: *const c_char) -> bool {
    clear_last_error();
    webview::evaluate_javascript(id, &str_from_ptr(js))
}

#[no_mangle]
pub extern "C" fn electrobun_dispatch_host_webview_event(id: u32, event_name: *const c_char, detail: *const c_char) -> bool {
    clear_last_error();
    webview::dispatch_host_webview_event(id, &str_from_ptr(event_name), &str_from_ptr(detail))
}

#[no_mangle]
pub extern "C" fn electrobun_clear_webview_host_transport(id: u32) -> bool {
    clear_last_error();
    webview::clear_host_transport(id)
}

#[no_mangle]
pub extern "C" fn electrobun_send_host_message_to_webview_via_transport(id: u32, message: *const c_char) -> bool {
    clear_last_error();
    webview::send_message_to_webview(id, &str_from_ptr(message))
}

#[no_mangle]
pub extern "C" fn electrobun_send_internal_message_to_webview(id: u32, message: *const c_char) -> bool {
    clear_last_error();
    webview::send_internal_message(id, &str_from_ptr(message))
}

#[no_mangle]
pub extern "C" fn electrobun_webview_open_devtools(id: u32) -> bool {
    clear_last_error();
    webview::open_devtools(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_close_devtools(id: u32) -> bool {
    clear_last_error();
    webview::close_devtools(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_toggle_devtools(id: u32) -> bool {
    clear_last_error();
    webview::toggle_devtools(id)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_set_page_zoom(id: u32, zoom: f64) -> bool {
    clear_last_error();
    webview::set_page_zoom(id, zoom)
}

#[no_mangle]
pub extern "C" fn electrobun_webview_get_page_zoom(id: u32) -> f64 {
    clear_last_error();
    webview::get_page_zoom(id)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WGPU VIEW (16)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_create_wgpu_view(window_id: u32, x: f64, y: f64, width: f64, height: f64, auto_resize: bool, start_transparent: bool, start_passthrough: bool) -> u32 {
    clear_last_error();
    let options = types::WgpuViewOptions {
        window_id,
        bounds: types::Rect { x, y, width, height },
        auto_resize,
        start_transparent,
        start_passthrough,
    };
    match wgpu::create_wgpu_view(options) {
        Ok(id) => id,
        Err(e) => { set_last_error(&e.to_string()); 0 }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_wgpu_view_pointer(_id: u32) -> *mut std::ffi::c_void {
    clear_last_error();
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn electrobun_set_wgpu_view_frame(id: u32, x: f64, y: f64, width: f64, height: f64) -> bool {
    clear_last_error();
    wgpu::set_wgpu_view_bounds(id, x, y, width, height)
}

#[no_mangle]
pub extern "C" fn electrobun_resize_wgpu_view(id: u32, width: f64, height: f64) -> bool {
    clear_last_error();
    wgpu::resize_wgpu_view(id, width, height)
}

#[no_mangle]
pub extern "C" fn electrobun_set_wgpu_view_transparent(id: u32, transparent: bool) -> bool {
    clear_last_error();
    wgpu::set_wgpu_view_transparent(id, transparent)
}

#[no_mangle]
pub extern "C" fn electrobun_set_wgpu_view_passthrough(id: u32, passthrough: bool) -> bool {
    clear_last_error();
    wgpu::set_wgpu_view_passthrough(id, passthrough)
}

#[no_mangle]
pub extern "C" fn electrobun_set_wgpu_view_hidden(id: u32, hidden: bool) -> bool {
    clear_last_error();
    wgpu::set_wgpu_view_hidden(id, hidden)
}

#[no_mangle]
pub extern "C" fn electrobun_remove_wgpu_view(id: u32) -> bool {
    clear_last_error();
    wgpu::remove_wgpu_view(id)
}

#[no_mangle]
pub extern "C" fn electrobun_get_wgpu_view_native_handle(id: u32) -> u64 {
    clear_last_error();
    wgpu::get_wgpu_view_native_handle(id).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn electrobun_run_wgpu_view_test(id: u32) -> bool {
    clear_last_error();
    wgpu::run_wgpu_view_test(id)
}

#[no_mangle]
pub extern "C" fn electrobun_toggle_wgpu_view_test_shader(id: u32) -> bool {
    clear_last_error();
    wgpu::toggle_wgpu_view_test_shader(id)
}

#[no_mangle]
pub extern "C" fn electrobun_wgpu_create_surface_for_view(id: u32) -> u32 {
    clear_last_error();
    wgpu::wgpu_create_surface_for_view(id)
}

#[no_mangle]
pub extern "C" fn electrobun_wgpu_create_adapter_device_main_thread(id: u32) -> u32 {
    clear_last_error();
    wgpu::wgpu_create_adapter_device_main_thread(id)
}

#[no_mangle]
pub extern "C" fn electrobun_wgpu_surface_configure_main_thread(id: u32) -> u32 {
    clear_last_error();
    wgpu::wgpu_surface_configure_main_thread(id)
}

#[no_mangle]
pub extern "C" fn electrobun_wgpu_surface_get_current_texture_main_thread(id: u32) -> u32 {
    clear_last_error();
    wgpu::wgpu_surface_get_current_texture_main_thread(id)
}

#[no_mangle]
pub extern "C" fn electrobun_wgpu_surface_present_main_thread(id: u32) -> u32 {
    clear_last_error();
    wgpu::wgpu_surface_present_main_thread(id)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAY (8)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_create_tray(image: *const c_char, title: *const c_char) -> u32 {
    clear_last_error();
    let image_str = str_from_ptr(image);
    let title_str = str_from_ptr(title);
    match tray::create_tray(&image_str, &title_str) {
        Ok(id) => id,
        Err(e) => { set_last_error(&e.to_string()); 0 }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_show_tray(id: u32) -> bool {
    clear_last_error();
    tray::show_tray(id)
}

#[no_mangle]
pub extern "C" fn electrobun_hide_tray(id: u32) -> bool {
    clear_last_error();
    tray::hide_tray(id)
}

#[no_mangle]
pub extern "C" fn electrobun_set_tray_title(id: u32, title: *const c_char) -> bool {
    clear_last_error();
    tray::set_tray_title(id, &str_from_ptr(title))
}

#[no_mangle]
pub extern "C" fn electrobun_set_tray_image(id: u32, image: *const c_char) -> bool {
    clear_last_error();
    tray::set_tray_image(id, &str_from_ptr(image))
}

#[no_mangle]
pub extern "C" fn electrobun_set_tray_menu(id: u32, menu_json: *const c_char) -> bool {
    clear_last_error();
    tray::set_tray_menu(id, &str_from_ptr(menu_json))
}

#[no_mangle]
pub extern "C" fn electrobun_remove_tray(id: u32) -> bool {
    clear_last_error();
    tray::destroy_tray(id)
}

#[no_mangle]
pub extern "C" fn electrobun_get_tray_bounds(id: u32, out_x: *mut f64, out_y: *mut f64, out_width: *mut f64, out_height: *mut f64) -> bool {
    clear_last_error();
    if out_x.is_null() || out_y.is_null() || out_width.is_null() || out_height.is_null() {
        return false;
    }
    match tray::get_tray_bounds(id) {
        Some((x, y, w, h)) => {
            unsafe { *out_x = x; *out_y = y; *out_width = w; *out_height = h; }
            true
        }
        None => false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MENU / SYSTEM DIALOGS (6)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_set_application_menu(menu_json: *const c_char) -> bool {
    clear_last_error();
    dialog::set_application_menu(&str_from_ptr(menu_json))
}

#[no_mangle]
pub extern "C" fn electrobun_show_context_menu(menu_json: *const c_char) -> bool {
    clear_last_error();
    dialog::show_context_menu(&str_from_ptr(menu_json))
}

#[no_mangle]
pub extern "C" fn electrobun_open_file_dialog(
    title: *const c_char,
    default_path: *const c_char,
    filters_json: *const c_char,
    multi: bool,
) -> *mut c_char {
    clear_last_error();
    let title_str = str_from_ptr(title);
    let path_str = str_from_ptr(default_path);
    // Parse filters from JSON: [["description", ["ext1","ext2"]]]
    let filters = vec![("All Files".to_string(), vec!["*".to_string()])];

    match dialog::open_file_dialog(&title_str, &path_str, &filters, multi) {
        Ok(paths) => {
            let json = serde_json::json!(paths).to_string();
            CString::new(json).map(|s| s.into_raw()).unwrap_or(ptr::null_mut())
        }
        Err(e) => {
            set_last_error(&e.to_string());
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_show_message_box(
    title: *const c_char,
    message: *const c_char,
    kind: u32,
) -> u32 {
    clear_last_error();
    let kind_enum = match kind {
        0 => dialog::MessageBoxKind::Info,
        1 => dialog::MessageBoxKind::Warning,
        2 => dialog::MessageBoxKind::Error,
        3 => dialog::MessageBoxKind::Question,
        _ => dialog::MessageBoxKind::Info,
    };
    match dialog::show_message_box(&str_from_ptr(title), &str_from_ptr(message), kind_enum) {
        dialog::MessageBoxResult::Ok => 1,
        dialog::MessageBoxResult::Cancel => 0,
    }
}

#[no_mangle]
pub extern "C" fn electrobun_move_to_trash(path: *const c_char) -> bool {
    clear_last_error();
    match dialog::move_to_trash(&str_from_ptr(path)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_show_item_in_folder(path: *const c_char) -> bool {
    clear_last_error();
    match dialog::show_item_in_folder(&str_from_ptr(path)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FILE / URL (2)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_open_external(url: *const c_char) -> bool {
    clear_last_error();
    match file_ops::open_external(&str_from_ptr(url)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_open_path(path: *const c_char) -> bool {
    clear_last_error();
    match file_ops::open_path(&str_from_ptr(path)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NOTIFICATIONS / DOCK (3)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_show_notification(title: *const c_char, body: *const c_char, icon_path: *const c_char) -> bool {
    clear_last_error();
    let icon = opt_str_from_ptr(icon_path);
    match notifications::show_notification(&str_from_ptr(title), &str_from_ptr(body), icon.as_deref()) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_set_dock_icon_visible(visible: bool) -> bool {
    clear_last_error();
    notifications::set_dock_icon_visible(visible)
}

#[no_mangle]
pub extern "C" fn electrobun_is_dock_icon_visible() -> bool {
    clear_last_error();
    notifications::is_dock_icon_visible()
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLIPBOARD (6)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_clipboard_read_text() -> *mut c_char {
    clear_last_error();
    match clipboard::read_text() {
        Ok(text) => CString::new(text).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
        Err(e) => { set_last_error(&e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_clipboard_write_text(text: *const c_char) -> bool {
    clear_last_error();
    match clipboard::write_text(&str_from_ptr(text)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_clipboard_read_image() -> *mut c_char {
    clear_last_error();
    match clipboard::read_image_base64() {
        Ok(b64) => CString::new(b64).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
        Err(e) => { set_last_error(&e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_clipboard_write_image(base64_data: *const c_char) -> bool {
    clear_last_error();
    match clipboard::write_image_base64(&str_from_ptr(base64_data)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_clipboard_clear() -> bool {
    clear_last_error();
    match clipboard::clear() {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_clipboard_available_formats() -> *mut c_char {
    clear_last_error();
    let formats = clipboard::available_formats();
    let json = serde_json::json!(formats).to_string();
    CString::new(json).map(|s| s.into_raw()).unwrap_or(ptr::null_mut())
}

// ═══════════════════════════════════════════════════════════════════════════════
// DISPLAY / INPUT (5)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_get_primary_display() -> *mut c_char {
    clear_last_error();
    match display::get_primary_display() {
        Ok(d) => {
            let json = serde_json::json!({
                "id": d.id,
                "bounds": { "x": d.bounds.x, "y": d.bounds.y, "width": d.bounds.width, "height": d.bounds.height },
                "workArea": { "x": d.work_area.x, "y": d.work_area.y, "width": d.work_area.width, "height": d.work_area.height },
                "scaleFactor": d.scale_factor,
                "isPrimary": d.is_primary,
            }).to_string();
            CString::new(json).map(|s| s.into_raw()).unwrap_or(ptr::null_mut())
        }
        Err(e) => { set_last_error(&e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_all_displays() -> *mut c_char {
    clear_last_error();
    match display::get_all_displays() {
        Ok(displays) => {
            let list: Vec<serde_json::Value> = displays.iter().map(|d| serde_json::json!({
                "id": d.id,
                "bounds": { "x": d.bounds.x, "y": d.bounds.y, "width": d.bounds.width, "height": d.bounds.height },
                "workArea": { "x": d.work_area.x, "y": d.work_area.y, "width": d.work_area.width, "height": d.work_area.height },
                "scaleFactor": d.scale_factor,
                "isPrimary": d.is_primary,
            })).collect();
            let json = serde_json::json!(list).to_string();
            CString::new(json).map(|s| s.into_raw()).unwrap_or(ptr::null_mut())
        }
        Err(e) => { set_last_error(&e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_cursor_screen_point(out_x: *mut f64, out_y: *mut f64) -> bool {
    clear_last_error();
    if out_x.is_null() || out_y.is_null() { return false; }
    match display::get_cursor_screen_point() {
        Ok((x, y)) => {
            unsafe { *out_x = x; *out_y = y; }
            true
        }
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_get_mouse_buttons() -> u32 {
    clear_last_error();
    display::get_mouse_buttons()
}

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

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL SHORTCUTS (5)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_set_global_shortcut_callback(_callback: Option<unsafe extern "C" fn(*const c_char)>) {
    clear_last_error();
    // Store shortcut callback
}

#[no_mangle]
pub extern "C" fn electrobun_register_global_shortcut(shortcut: *const c_char) -> bool {
    clear_last_error();
    shortcuts::register_shortcut(&str_from_ptr(shortcut))
}

#[no_mangle]
pub extern "C" fn electrobun_unregister_global_shortcut(shortcut: *const c_char) -> bool {
    clear_last_error();
    shortcuts::unregister_shortcut(&str_from_ptr(shortcut))
}

#[no_mangle]
pub extern "C" fn electrobun_unregister_all_global_shortcuts() -> bool {
    clear_last_error();
    shortcuts::unregister_all()
}

#[no_mangle]
pub extern "C" fn electrobun_is_global_shortcut_registered(shortcut: *const c_char) -> bool {
    clear_last_error();
    shortcuts::is_shortcut_registered(&str_from_ptr(shortcut))
}

// ═══════════════════════════════════════════════════════════════════════════════
// SESSION / COOKIES (5)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub extern "C" fn electrobun_session_get_cookies(url: *const c_char) -> *mut c_char {
    clear_last_error();
    match session::get_cookies(&str_from_ptr(url)) {
        Ok(json) => CString::new(json).map(|s| s.into_raw()).unwrap_or(ptr::null_mut()),
        Err(e) => { set_last_error(&e.to_string()); ptr::null_mut() }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_session_set_cookie(
    url: *const c_char, name: *const c_char, value: *const c_char,
    domain: *const c_char, path: *const c_char,
    secure: bool, http_only: bool, max_age: i64,
) -> bool {
    clear_last_error();
    let domain_opt = opt_str_from_ptr(domain);
    let path_opt = opt_str_from_ptr(path);
    match session::set_cookie(
        &str_from_ptr(url), &str_from_ptr(name), &str_from_ptr(value),
        domain_opt.as_deref(), path_opt.as_deref(),
        secure, http_only, Some(max_age),
    ) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_session_remove_cookie(url: *const c_char, name: *const c_char) -> bool {
    clear_last_error();
    match session::remove_cookie(&str_from_ptr(url), &str_from_ptr(name)) {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_session_clear_cookies() -> bool {
    clear_last_error();
    match session::clear_cookies() {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}

#[no_mangle]
pub extern "C" fn electrobun_session_clear_storage_data() -> bool {
    clear_last_error();
    match session::clear_storage_data() {
        Ok(_) => true,
        Err(e) => { set_last_error(&e.to_string()); false }
    }
}
