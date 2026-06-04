//! WGPU GPU view module

use crate::error::ElectrobunError;
use crate::types::{Rect, WgpuViewOptions, WgpuViewState};
use crate::WGPU_VIEW_REGISTRY;

/// Create a new WGPU view
///
/// # Errors
///
/// Returns [`ElectrobunError`] if platform-specific WGPU view creation fails.
pub fn create_wgpu_view(options: WgpuViewOptions) -> Result<u32, ElectrobunError> {
    let id = crate::next_wgpu_view_id();
    
    let state = WgpuViewState {
        id,
        window_id: options.window_id,
        bounds: options.bounds,
        transparent: options.start_transparent,
        visible: true,
    };
    
    let mut registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.insert(id, state);
    
    // Platform-specific WGPU view creation
    #[cfg(target_os = "macos")]
    create_macos_wgpu_view(id, &options)?;
    
    #[cfg(target_os = "windows")]
    create_windows_wgpu_view(id, &options)?;
    
    #[cfg(target_os = "linux")]
    create_linux_wgpu_view(id, &options)?;
    
    Ok(id)
}

/// Destroy a WGPU view
pub fn destroy_wgpu_view(id: u32) -> bool {
    let mut registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    
    if registry.remove(&id).is_some() {
        #[cfg(target_os = "macos")]
        destroy_macos_wgpu_view(id);
        
        #[cfg(target_os = "windows")]
        destroy_windows_wgpu_view(id);
        
        #[cfg(target_os = "linux")]
        destroy_linux_wgpu_view(id);
        
        true
    } else {
        false
    }
}

/// Set WGPU view bounds
pub fn set_wgpu_view_bounds(id: u32, x: f64, y: f64, width: f64, height: f64) -> bool {
    let mut registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    
    if let Some(state) = registry.get_mut(&id) {
        state.bounds = Rect { x, y, width, height };
        
        #[cfg(target_os = "macos")]
        set_macos_wgpu_view_bounds(id, x, y, width, height);
        
        #[cfg(target_os = "windows")]
        set_windows_wgpu_view_bounds(id, x, y, width, height);
        
        #[cfg(target_os = "linux")]
        set_linux_wgpu_view_bounds(id, x, y, width, height);
        
        true
    } else {
        false
    }
}

/// Get WGPU view bounds
pub fn get_wgpu_view_bounds(id: u32) -> Option<Rect> {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.get(&id).map(|state| state.bounds)
}

/// Show/hide WGPU view
pub fn set_wgpu_view_visible(id: u32, visible: bool) -> bool {
    let mut registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    
    if let Some(state) = registry.get_mut(&id) {
        state.visible = visible;
        
        #[cfg(target_os = "macos")]
        set_macos_wgpu_view_visible(id, visible);
        
        #[cfg(target_os = "windows")]
        set_windows_wgpu_view_visible(id, visible);
        
        #[cfg(target_os = "linux")]
        set_linux_wgpu_view_visible(id, visible);
        
        true
    } else {
        false
    }
}

/// Set WGPU view transparent
pub fn set_wgpu_view_transparent(id: u32, transparent: bool) -> bool {
    let mut registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    
    if let Some(state) = registry.get_mut(&id) {
        state.transparent = transparent;
        
        #[cfg(target_os = "macos")]
        set_macos_wgpu_view_transparent(id, transparent);
        
        #[cfg(target_os = "windows")]
        set_windows_wgpu_view_transparent(id, transparent);
        
        #[cfg(target_os = "linux")]
        set_linux_wgpu_view_transparent(id, transparent);
        
        true
    } else {
        false
    }
}

/// Get WGPU view native handle
pub fn get_wgpu_view_native_handle(id: u32) -> Option<u64> {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if registry.get(&id).is_some() {
        // Native handle requires platform-specific implementation
        // Returns a dummy handle for now
        Some(id as u64)
    } else {
        None
    }
}

/// Run test shader on WGPU view
pub fn run_wgpu_view_test(id: u32) -> bool {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.get(&id).is_some()
}

/// Toggle test shader on WGPU view
pub fn toggle_wgpu_view_test_shader(id: u32) -> bool {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.get(&id).is_some()
}

/// Remove a WGPU view (alias for destroy)
pub fn remove_wgpu_view(id: u32) -> bool {
    destroy_wgpu_view(id)
}

/// Resize WGPU view
pub fn resize_wgpu_view(id: u32, width: f64, height: f64) -> bool {
    let mut registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(state) = registry.get_mut(&id) {
        state.bounds.width = width;
        state.bounds.height = height;
        true
    } else {
        false
    }
}

/// Set WGPU view passthrough (click-through)
pub fn set_wgpu_view_passthrough(_id: u32, _passthrough: bool) -> bool {
    // Passthrough would require native window manipulation
    true
}

/// Set WGPU view hidden
pub fn set_wgpu_view_hidden(id: u32, hidden: bool) -> bool {
    set_wgpu_view_visible(id, !hidden)
}

/// Create wgpu surface for the view (stub - needs window handle from event loop)
pub fn wgpu_create_surface_for_view(id: u32) -> u32 {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if registry.get(&id).is_some() {
        1 // Success indicator
    } else {
        0
    }
}

/// Create wgpu adapter and device (stub)
pub fn wgpu_create_adapter_device_main_thread(id: u32) -> u32 {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if registry.get(&id).is_some() {
        1 // Success indicator
    } else {
        0
    }
}

/// Configure wgpu surface (stub)
pub fn wgpu_surface_configure_main_thread(id: u32) -> u32 {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if registry.get(&id).is_some() {
        1 // Success indicator
    } else {
        0
    }
}

/// Get current texture from wgpu surface (stub)
pub fn wgpu_surface_get_current_texture_main_thread(id: u32) -> u32 {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if registry.get(&id).is_some() {
        1 // Success indicator
    } else {
        0
    }
}

/// Present the wgpu surface (stub)
pub fn wgpu_surface_present_main_thread(id: u32) -> u32 {
    let registry = WGPU_VIEW_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if registry.get(&id).is_some() {
        1 // Success indicator
    } else {
        0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platform-specific implementations (stubs)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
fn create_macos_wgpu_view(_id: u32, _options: &WgpuViewOptions) -> Result<(), ElectrobunError> {
    // TODO: Implement with CAMetalLayer
    Ok(())
}

#[cfg(target_os = "macos")]
fn destroy_macos_wgpu_view(_id: u32) {}

#[cfg(target_os = "macos")]
fn set_macos_wgpu_view_bounds(_id: u32, _x: f64, _y: f64, _width: f64, _height: f64) {}

#[cfg(target_os = "macos")]
fn set_macos_wgpu_view_visible(_id: u32, _visible: bool) {}

#[cfg(target_os = "macos")]
fn set_macos_wgpu_view_transparent(_id: u32, _transparent: bool) {}

#[cfg(target_os = "windows")]
fn create_windows_wgpu_view(_id: u32, _options: &WgpuViewOptions) -> Result<(), ElectrobunError> {
    // TODO: Implement with SwapChainPanel
    Ok(())
}

#[cfg(target_os = "windows")]
fn destroy_windows_wgpu_view(_id: u32) {}

#[cfg(target_os = "windows")]
fn set_windows_wgpu_view_bounds(_id: u32, _x: f64, _y: f64, _width: f64, _height: f64) {}

#[cfg(target_os = "windows")]
fn set_windows_wgpu_view_visible(_id: u32, _visible: bool) {}

#[cfg(target_os = "windows")]
fn set_windows_wgpu_view_transparent(_id: u32, _transparent: bool) {}

#[cfg(target_os = "linux")]
fn create_linux_wgpu_view(_id: u32, _options: &WgpuViewOptions) -> Result<(), ElectrobunError> {
    // TODO: Implement with GTK GLArea or EGL surface
    Ok(())
}

#[cfg(target_os = "linux")]
fn destroy_linux_wgpu_view(_id: u32) {}

#[cfg(target_os = "linux")]
fn set_linux_wgpu_view_bounds(_id: u32, _x: f64, _y: f64, _width: f64, _height: f64) {}

#[cfg(target_os = "linux")]
fn set_linux_wgpu_view_visible(_id: u32, _visible: bool) {}

#[cfg(target_os = "linux")]
fn set_linux_wgpu_view_transparent(_id: u32, _transparent: bool) {}
