// Display and monitor information

use crate::types::Rect;
use crate::error::ElectrobunError;

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
pub fn get_primary_display() -> Result<DisplayInfo, ElectrobunError> {
    let displays = get_all_displays()?;
    displays.into_iter().find(|d| d.is_primary)
        .ok_or_else(|| ElectrobunError::OperationFailed("No primary display found".to_string()))
}

/// Get all displays
pub fn get_all_displays() -> Result<Vec<DisplayInfo>, ElectrobunError> {
    let mut displays = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Graphics::Gdi::*;
        unsafe {
            let mut index: u32 = 0;
            loop {
                let mut device_name: [u16; 32] = [0u16; 32];
                let mut device_flags: u32 = 0;

                let mut disp_mode: DEVMODEW = std::mem::zeroed();
                disp_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

                if EnumDisplaySettingsW(None, ENUM_DISPLAY_SETTINGS_MODE(index), &mut disp_mode).as_bool() {
                    let id = index as i64;
                    let bounds = Rect {
                        x: disp_mode.dmPosition.x as f64,
                        y: disp_mode.dmPosition.y as f64,
                        width: disp_mode.dmPelsWidth as f64,
                        height: disp_mode.dmPelsHeight as f64,
                    };

                    let is_primary = (disp_mode.dmFields & DM_DISPLAYFIXEDOUTPUT) != 0 || index == 0;

                    displays.push(DisplayInfo {
                        id,
                        bounds,
                        work_area: bounds, // TODO: get actual work area
                        scale_factor: 1.0,
                        is_primary,
                        name: String::new(),
                    });

                    index += 1;
                } else {
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Use winit's monitor handle info as fallback
        // For now, return a basic display
        displays.push(DisplayInfo {
            id: 0,
            bounds: Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 },
            work_area: Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1050.0 },
            scale_factor: 2.0,
            is_primary: true,
            name: "Primary Display".to_string(),
        });
    }

    #[cfg(target_os = "linux")]
    {
        // X11 display enumeration fallback
        displays.push(DisplayInfo {
            id: 0,
            bounds: Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 },
            work_area: Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1040.0 },
            scale_factor: 1.0,
            is_primary: true,
            name: "Primary Display".to_string(),
        });
    }

    if displays.is_empty() {
        // Fallback to single display
        displays.push(DisplayInfo {
            id: 0,
            bounds: Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 },
            work_area: Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1040.0 },
            scale_factor: 1.0,
            is_primary: true,
            name: "Default Display".to_string(),
        });
    }

    Ok(displays)
}

/// Get cursor screen position
pub fn get_cursor_screen_point() -> Result<(f64, f64), ElectrobunError> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        use windows::Win32::Foundation::POINT;

        unsafe {
            let mut pt: POINT = std::mem::zeroed();
            if GetCursorPos(&mut pt).as_bool() {
                return Ok((pt.x as f64, pt.y as f64));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Could use CoreGraphics APIs, but for now return generic
        return Ok((0.0, 0.0));
    }

    #[cfg(target_os = "linux")]
    {
        return Ok((0.0, 0.0));
    }

    Ok((0.0, 0.0))
}

/// Get mouse buttons state (returns bitmask)
pub fn get_mouse_buttons() -> u32 {
    // TODO: Implement platform-specific mouse button query
    0
}
