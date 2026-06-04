//! Electrobun Core — Rust FFI Library
//!
//! Provides the FFI interface for the Electrobun framework.
//! All functions are #[no_mangle] pub extern "C" for C ABI compatibility with Bun/TypeScript.
//! Safety docs omitted for FFI exports — callers must ensure valid pointers per C ABI.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::expect_used)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::type_complexity)]
#![allow(non_upper_case_globals)]
#![allow(unused)]

pub mod api;
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
    let mut id = NEXT_WINDOW_ID.lock().unwrap_or_else(|e| e.into_inner());
    let result = *id;
    *id += 1;
    result
}

fn next_webview_id() -> u32 {
    let mut id = NEXT_WEBVIEW_ID.lock().unwrap_or_else(|e| e.into_inner());
    let result = *id;
    *id += 1;
    result
}

fn next_wgpu_view_id() -> u32 {
    let mut id = NEXT_WGPU_VIEW_ID.lock().unwrap_or_else(|e| e.into_inner());
    let result = *id;
    *id += 1;
    result
}

fn next_tray_id() -> u32 {
    let mut id = NEXT_TRAY_ID.lock().unwrap_or_else(|e| e.into_inner());
    let result = *id;
    *id += 1;
    result
}

fn clear_last_error() {
    let mut err = LAST_ERROR.lock().unwrap_or_else(|e| e.into_inner());
    *err = None;
}

fn set_last_error(msg: &str) {
    let mut err = LAST_ERROR.lock().unwrap_or_else(|e| e.into_inner());
    *err = Some(CString::new(msg).unwrap_or_else(|_| {
        CString::new("Unknown error").expect("static string has no null bytes")
    }));
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
pub unsafe extern "C" fn electrobun_core_last_error() -> *const c_char {
    let err = LAST_ERROR.lock().unwrap_or_else(|e| e.into_inner());
    match err.as_ref() {
        Some(cstr) => cstr.as_ptr(),
        None => ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn electrobun_free_core_string(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            let _ = CString::from_raw(value);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn electrobun_pop_next_queued_host_message(
    out_webview_id: *mut u32,
) -> *mut c_char {
    clear_last_error();
    if out_webview_id.is_null() {
        return ptr::null_mut();
    }
    match webview::pop_next_message() {
        Some((webview_id, message)) => {
            unsafe {
                *out_webview_id = webview_id;
            }
            CString::new(message)
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut())
        },
        None => {
            unsafe {
                *out_webview_id = 0;
            }
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn electrobun_get_host_message_wakeup_read_fd() -> std::os::raw::c_int {
    clear_last_error();
    transport::get_wakeup_fd().unwrap_or(-1)
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIFECYCLE (6)
// ═══════════════════════════════════════════════════════════════════════════════

#[no_mangle]
pub unsafe extern "C" fn electrobun_configure_webview_runtime(
    rpc_port: u32,
    preload_script: *const c_char,
    preload_script_sandboxed: *const c_char,
) -> bool {
    clear_last_error();
    let preload = opt_str_from_ptr(preload_script);
    let preload_sandboxed = opt_str_from_ptr(preload_script_sandboxed);
    let mut state = webview::WEBVIEW_RUNTIME_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    state.rpc_port = rpc_port;
    state.preload_script = preload;
    state.preload_script_sandboxed = preload_sandboxed;
    state.configured = true;
    true
}

#[no_mangle]
pub unsafe extern "C" fn electrobun_init_webview_runtime(
    preload_script: *const c_char,
    preload_script_sandboxed: *const c_char,
) -> bool {
    clear_last_error();
    let preload = opt_str_from_ptr(preload_script);
    let preload_sandboxed = opt_str_from_ptr(preload_script_sandboxed);
    let mut state = webview::WEBVIEW_RUNTIME_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    state.preload_script = preload;
    state.preload_script_sandboxed = preload_sandboxed;
    state.configured = true;
    true
}

/// Initialize the event loop channel before the worker starts.
/// Call this from the launcher BEFORE creating the Worker to avoid
/// a race where the worker creates a window before the event loop is ready.
#[no_mangle]
pub unsafe extern "C" fn electrobun_init_event_loop() {
    clear_last_error();
    window::init_event_loop();
}

/// Set the views root directory for the `views://` custom protocol
#[no_mangle]
pub unsafe extern "C" fn electrobun_set_views_root(path: *const c_char) {
    clear_last_error();
    let mut state = webview::WEBVIEW_RUNTIME_STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    state.views_root = opt_str_from_ptr(path);
}

#[no_mangle]
pub unsafe extern "C" fn electrobun_core_run_main_thread(
    _identifier: *const std::os::raw::c_char,
    _name: *const std::os::raw::c_char,
    _channel: *const std::os::raw::c_char,
    _flag: std::os::raw::c_int,
) -> std::os::raw::c_int {
    clear_last_error();
    // 先初始化 event loop 通道，确保 Worker 发命令时有人消费
    window::init_event_loop();
    window::run_blocking();
    0
}

/// Set URL open handler (for macOS open-url events)
#[no_mangle]
pub unsafe extern "C" fn electrobun_set_url_open_handler(
    _handler: Option<unsafe extern "C" fn(*const c_char)>,
) {
    // TODO: Store and invoke URL open handler
}

/// Set app reopen handler (for macOS reopen events)
#[no_mangle]
pub unsafe extern "C" fn electrobun_set_app_reopen_handler(
    _handler: Option<unsafe extern "C" fn()>,
) {
    // TODO: Store and invoke app reopen handler
}

/// Set quit requested handler
#[no_mangle]
pub unsafe extern "C" fn electrobun_set_quit_requested_handler(
    _handler: Option<unsafe extern "C" fn()>,
) {
    // TODO: Store and invoke quit requested handler
}

/// Set exit on last window closed
pub static mut EXIT_ON_LAST_WINDOW_CLOSED: bool = true;

// EVENT LOOP CONTROL

#[no_mangle]
pub unsafe extern "C" fn electrobun_wait_for_shutdown_complete() {
    // The join handle in window::shutdown already waits
}

// ============================================================
// NOTE: All other FFI functions migrated to #[eden_ipc] traits
// ============================================================