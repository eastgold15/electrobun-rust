// FFI symbol definitions for core and native wrapper dlopen calls.
import { FFIType } from "bun:ffi";

export const CORE_SYMBOLS = {
  electrobun_init_core: { args: [], returns: FFIType.bool },
  electrobun_core_last_error: { args: [], returns: FFIType.cstring },
  electrobun_free_core_string: { args: [FFIType.ptr], returns: FFIType.void },
  electrobun_pop_next_queued_host_message: {
    args: [FFIType.ptr],
    returns: FFIType.ptr,
  },
  electrobun_get_host_message_wakeup_read_fd: {
    args: [],
    returns: FFIType.i32,
  },
  electrobun_configure_webview_runtime: {
    args: [FFIType.u32, FFIType.cstring, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_init_webview_runtime: {
    args: [FFIType.cstring, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_set_views_root: { args: [FFIType.cstring], returns: FFIType.void },
  electrobun_core_run_main_thread: {
    args: [FFIType.cstring, FFIType.cstring, FFIType.cstring, FFIType.i32],
    returns: FFIType.i32,
  },
  electrobun_set_exit_on_last_window_closed: {
    args: [FFIType.bool],
    returns: FFIType.void,
  },
  electrobun_quit_gracefully: { args: [], returns: FFIType.void },
  electrobun_stop_event_loop: { args: [], returns: FFIType.void },
  electrobun_wait_for_shutdown_complete: { args: [], returns: FFIType.void },
  electrobun_force_exit: { args: [FFIType.i32], returns: FFIType.void },
  electrobun_set_url_open_handler: {
    args: [FFIType.function],
    returns: FFIType.void,
  },
  electrobun_set_app_reopen_handler: {
    args: [FFIType.function],
    returns: FFIType.void,
  },
  electrobun_set_quit_requested_handler: {
    args: [FFIType.function],
    returns: FFIType.void,
  },
  electrobun_get_window_style: {
    args: [FFIType.u32],
    returns: FFIType.cstring,
  },
  electrobun_create_window: {
    args: [
      FFIType.cstring,
      FFIType.f64,
      FFIType.f64,
      FFIType.f64,
      FFIType.f64,
      FFIType.u32,
    ],
    returns: FFIType.u32,
  },
  electrobun_get_window_pointer: { args: [FFIType.u32], returns: FFIType.ptr },
  electrobun_set_window_title: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_minimize_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_restore_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_is_window_minimized: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_maximize_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_unmaximize_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_is_window_maximized: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_set_window_fullscreen: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_is_window_fullscreen: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_set_window_always_on_top: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_is_window_always_on_top: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_set_window_visible_on_all_workspaces: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_is_window_visible_on_all_workspaces: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_set_window_button_position: {
    args: [FFIType.u32, FFIType.f64, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_show_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_activate_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_hide_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_close_window: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_set_window_position: {
    args: [FFIType.u32, FFIType.f64, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_set_window_size: {
    args: [FFIType.u32, FFIType.f64, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_set_window_frame: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_get_window_frame: {
    args: [FFIType.u32, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
    returns: FFIType.bool,
  },
  electrobun_begin_window_move: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_end_window_move: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_create_webview: {
    args: [
      FFIType.u32,
      FFIType.cstring,
      FFIType.cstring,
      FFIType.cstring,
      FFIType.bool,
      FFIType.bool,
    ],
    returns: FFIType.u32,
  },
  electrobun_get_webview_pointer: { args: [FFIType.u32], returns: FFIType.ptr },
  electrobun_resize_webview: {
    args: [FFIType.u32, FFIType.f64, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_load_url_in_webview: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_load_html_in_webview: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_update_preload_script_to_webview: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_webview_can_go_back: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_webview_can_go_forward: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_webview_go_back: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_webview_go_forward: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_webview_reload: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_webview_remove: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_set_webview_html_content: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_webview_set_transparent: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_webview_set_passthrough: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_webview_set_hidden: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_set_webview_navigation_rules: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_webview_find_in_page: {
    args: [FFIType.u32, FFIType.cstring, FFIType.bool, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_webview_stop_find: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },

  electrobun_evaluate_javascript_with_no_completion: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_dispatch_host_webview_event: {
    args: [FFIType.u32, FFIType.cstring, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_clear_webview_host_transport: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_send_host_message_to_webview_via_transport: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_send_internal_message_to_webview: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_webview_open_devtools: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_webview_close_devtools: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_webview_toggle_devtools: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_webview_set_page_zoom: {
    args: [FFIType.u32, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_webview_get_page_zoom: {
    args: [FFIType.u32],
    returns: FFIType.f64,
  },
  electrobun_create_wgpu_view: {
    args: [
      FFIType.u32,
      FFIType.f64,
      FFIType.f64,
      FFIType.f64,
      FFIType.f64,
      FFIType.bool,
      FFIType.bool,
      FFIType.bool,
    ],
    returns: FFIType.u32,
  },
  electrobun_get_wgpu_view_pointer: {
    args: [FFIType.u32],
    returns: FFIType.ptr,
  },
  electrobun_set_wgpu_view_frame: {
    args: [FFIType.u32, FFIType.f64, FFIType.f64, FFIType.f64, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_resize_wgpu_view: {
    args: [FFIType.u32, FFIType.f64, FFIType.f64],
    returns: FFIType.bool,
  },
  electrobun_set_wgpu_view_transparent: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_set_wgpu_view_passthrough: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_set_wgpu_view_hidden: {
    args: [FFIType.u32, FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_remove_wgpu_view: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_get_wgpu_view_native_handle: {
    args: [FFIType.u32],
    returns: FFIType.u64,
  },
  electrobun_run_wgpu_view_test: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_toggle_wgpu_view_test_shader: {
    args: [FFIType.u32],
    returns: FFIType.bool,
  },
  electrobun_wgpu_create_surface_for_view: {
    args: [FFIType.u32],
    returns: FFIType.u32,
  },
  electrobun_wgpu_create_adapter_device_main_thread: {
    args: [FFIType.u32],
    returns: FFIType.u32,
  },
  electrobun_wgpu_surface_configure_main_thread: {
    args: [FFIType.u32],
    returns: FFIType.u32,
  },
  electrobun_wgpu_surface_get_current_texture_main_thread: {
    args: [FFIType.u32],
    returns: FFIType.u32,
  },
  electrobun_wgpu_surface_present_main_thread: {
    args: [FFIType.u32],
    returns: FFIType.u32,
  },
  electrobun_create_tray: {
    args: [FFIType.cstring, FFIType.cstring],
    returns: FFIType.u32,
  },
  electrobun_show_tray: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_hide_tray: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_set_tray_title: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_set_tray_image: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_set_tray_menu: {
    args: [FFIType.u32, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_remove_tray: { args: [FFIType.u32], returns: FFIType.bool },
  electrobun_get_tray_bounds: {
    args: [FFIType.u32, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr],
    returns: FFIType.bool,
  },
  electrobun_set_application_menu: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_show_context_menu: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_open_file_dialog: {
    args: [FFIType.cstring, FFIType.cstring, FFIType.cstring, FFIType.bool],
    returns: FFIType.cstring,
  },
  electrobun_show_message_box: {
    args: [FFIType.cstring, FFIType.cstring, FFIType.u32],
    returns: FFIType.u32,
  },
  electrobun_move_to_trash: { args: [FFIType.cstring], returns: FFIType.bool },
  electrobun_show_item_in_folder: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_open_external: { args: [FFIType.cstring], returns: FFIType.bool },
  electrobun_open_path: { args: [FFIType.cstring], returns: FFIType.bool },
  electrobun_show_notification: {
    args: [FFIType.cstring, FFIType.cstring, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_set_dock_icon_visible: {
    args: [FFIType.bool],
    returns: FFIType.bool,
  },
  electrobun_is_dock_icon_visible: { args: [], returns: FFIType.bool },
  electrobun_clipboard_read_text: { args: [], returns: FFIType.cstring },
  electrobun_clipboard_write_text: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_clipboard_read_image: { args: [], returns: FFIType.cstring },
  electrobun_clipboard_write_image: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_clipboard_clear: { args: [], returns: FFIType.bool },
  electrobun_clipboard_available_formats: {
    args: [],
    returns: FFIType.cstring,
  },
  electrobun_get_primary_display: { args: [], returns: FFIType.cstring },
  electrobun_get_all_displays: { args: [], returns: FFIType.cstring },
  electrobun_get_cursor_screen_point: {
    args: [FFIType.ptr, FFIType.ptr],
    returns: FFIType.bool,
  },
  electrobun_get_mouse_buttons: { args: [], returns: FFIType.u32 },
  electrobun_get_platform: { args: [], returns: FFIType.cstring },
  electrobun_register_global_shortcut: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_unregister_global_shortcut: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_unregister_all_global_shortcuts: {
    args: [],
    returns: FFIType.bool,
  },
  electrobun_is_global_shortcut_registered: {
    args: [FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_session_get_cookies: {
    args: [FFIType.cstring],
    returns: FFIType.cstring,
  },
  electrobun_session_set_cookie: {
    args: [
      FFIType.cstring,
      FFIType.cstring,
      FFIType.cstring,
      FFIType.cstring,
      FFIType.cstring,
      FFIType.bool,
      FFIType.bool,
      FFIType.i64,
    ],
    returns: FFIType.bool,
  },
  electrobun_session_remove_cookie: {
    args: [FFIType.cstring, FFIType.cstring],
    returns: FFIType.bool,
  },
  electrobun_session_clear_cookies: { args: [], returns: FFIType.bool },
  electrobun_session_clear_storage_data: { args: [], returns: FFIType.bool },
} as const;
