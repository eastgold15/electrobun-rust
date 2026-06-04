//! Desktop notifications
//! Uses notify-rust crate

use crate::error::ElectrobunError;

/// Show a desktop notification
pub fn show_notification(
    title: &str,
    body: &str,
    icon_path: Option<&str>,
) -> Result<(), ElectrobunError> {
    let mut notification = notify_rust::Notification::new();
    notification.summary(title).body(body).appname("Electrobun");

    if let Some(icon) = icon_path {
        if !icon.is_empty() {
            notification.icon(icon);
        }
    }

    notification.show().map_err(|e| {
        ElectrobunError::OperationFailed(format!("Failed to show notification: {}", e))
    })?;

    Ok(())
}

/// Set dock icon visibility (macOS only)
pub fn set_dock_icon_visible(visible: bool) -> bool {
    #[cfg(target_os = "macos")]
    {
        // TODO: Use objc2 to set NSApp.setActivationPolicy
        let _ = visible;
        true
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = visible;
        true // No-op on non-macOS
    }
}

/// Check if dock icon is visible (macOS only)
pub fn is_dock_icon_visible() -> bool {
    true // Default to visible
}
