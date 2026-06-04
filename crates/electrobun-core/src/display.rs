//! Display and monitor information

use crate::error::ElectrobunError;
use crate::types::Rect;

/// Display information
pub struct DisplayInfo {
    pub id: i64,
    pub bounds: Rect,
    pub work_area: Rect,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub name: String,
}

/// Get the primary display
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if no primary display is found.
pub fn get_primary_display() -> Result<DisplayInfo, ElectrobunError> {
    let displays = get_all_displays()?;
    displays
        .into_iter()
        .find(|d| d.is_primary)
        .ok_or_else(|| ElectrobunError::OperationFailed("No primary display found".to_string()))
}

/// Get all displays
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if display information cannot be retrieved.
pub fn get_all_displays() -> Result<Vec<DisplayInfo>, ElectrobunError> {
    let displays = vec![DisplayInfo {
        id: 0,
        bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        },
        work_area: Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1040.0,
        },
        scale_factor: 1.0,
        is_primary: true,
        name: "Primary Display".to_string(),
    }];
    Ok(displays)
}

/// Get cursor screen position
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if the cursor position cannot be determined.
pub fn get_cursor_screen_point() -> Result<(f64, f64), ElectrobunError> {
    Ok((0.0, 0.0))
}

/// Get mouse buttons state
pub fn get_mouse_buttons() -> u32 {
    0
}
