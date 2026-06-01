//! System tray module

use crate::error::ElectrobunError;
use crate::types::{TrayOptions, TrayState};
use crate::TRAY_REGISTRY;

/// Create a new tray
pub fn create_tray(image: &str, title: &str) -> Result<u32, ElectrobunError> {
    let id = crate::next_tray_id();
    
    let state = TrayState {
        id,
        title: title.to_string(),
        image: image.to_string(),
        visible: true,
        handler: None,
    };
    
    let mut registry = TRAY_REGISTRY.lock().unwrap();
    registry.insert(id, state);
    
    // Platform-specific tray creation
    #[cfg(target_os = "macos")]
    create_macos_tray(id, image, title)?;
    
    #[cfg(target_os = "windows")]
    create_windows_tray(id, image, title)?;
    
    #[cfg(target_os = "linux")]
    create_linux_tray(id, image, title)?;
    
    Ok(id)
}

/// Destroy a tray
pub fn destroy_tray(id: u32) -> bool {
    let mut registry = TRAY_REGISTRY.lock().unwrap();
    
    if registry.remove(&id).is_some() {
        #[cfg(target_os = "macos")]
        destroy_macos_tray(id);
        
        #[cfg(target_os = "windows")]
        destroy_windows_tray(id);
        
        #[cfg(target_os = "linux")]
        destroy_linux_tray(id);
        
        true
    } else {
        false
    }
}

/// Set tray image
pub fn set_tray_image(id: u32, image: &str) -> bool {
    let mut registry = TRAY_REGISTRY.lock().unwrap();
    
    if let Some(state) = registry.get_mut(&id) {
        state.image = image.to_string();
        
        #[cfg(target_os = "macos")]
        set_macos_tray_image(id, image);
        
        #[cfg(target_os = "windows")]
        set_windows_tray_image(id, image);
        
        #[cfg(target_os = "linux")]
        set_linux_tray_image(id, image);
        
        true
    } else {
        false
    }
}

/// Set tray title
pub fn set_tray_title(id: u32, title: &str) -> bool {
    let mut registry = TRAY_REGISTRY.lock().unwrap();
    
    if let Some(state) = registry.get_mut(&id) {
        state.title = title.to_string();
        
        #[cfg(target_os = "macos")]
        set_macos_tray_title(id, title);
        
        #[cfg(target_os = "windows")]
        set_windows_tray_title(id, title);
        
        #[cfg(target_os = "linux")]
        set_linux_tray_title(id, title);
        
        true
    } else {
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platform-specific implementations (stubs)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
fn create_macos_tray(_id: u32, _image: &str, _title: &str) -> Result<(), ElectrobunError> {
    // TODO: Implement with NSStatusItem
    Ok(())
}

#[cfg(target_os = "macos")]
fn destroy_macos_tray(_id: u32) {}

#[cfg(target_os = "macos")]
fn set_macos_tray_image(_id: u32, _image: &str) {}

#[cfg(target_os = "macos")]
fn set_macos_tray_title(_id: u32, _title: &str) {}

#[cfg(target_os = "windows")]
fn create_windows_tray(_id: u32, _image: &str, _title: &str) -> Result<(), ElectrobunError> {
    // TODO: Implement with Shell_NotifyIcon
    Ok(())
}

#[cfg(target_os = "windows")]
fn destroy_windows_tray(_id: u32) {}

#[cfg(target_os = "windows")]
fn set_windows_tray_image(_id: u32, _image: &str) {}

#[cfg(target_os = "windows")]
fn set_windows_tray_title(_id: u32, _title: &str) {}

#[cfg(target_os = "linux")]
fn create_linux_tray(_id: u32, _image: &str, _title: &str) -> Result<(), ElectrobunError> {
    // TODO: Implement with AppIndicator
    Ok(())
}

#[cfg(target_os = "linux")]
fn destroy_linux_tray(_id: u32) {}

#[cfg(target_os = "linux")]
fn set_linux_tray_image(_id: u32, _image: &str) {}

#[cfg(target_os = "linux")]
fn set_linux_tray_title(_id: u32, _title: &str) {}
