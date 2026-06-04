//! System dialogs and native menus

use crate::error::ElectrobunError;

/// Open a file dialog (simplified - uses rfd if available)
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if the system file dialog cannot be opened.
pub fn open_file_dialog(
    title: &str,
    default_path: &str,
    _filters: &[(String, Vec<String>)],
    multi: bool,
) -> Result<Vec<String>, ElectrobunError> {
    let mut dialog = rfd::FileDialog::new().set_title(title);

    if !default_path.is_empty() {
        dialog = dialog.set_directory(default_path);
    }

    let result = if multi {
        dialog.pick_files().map(|files| {
            files
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        })
    } else {
        dialog
            .pick_file()
            .map(|f| vec![f.to_string_lossy().to_string()])
    };

    Ok(result.unwrap_or_default())
}

/// Show a message box
pub fn show_message_box(title: &str, message: &str, kind: MessageBoxKind) -> MessageBoxResult {
    let level = match kind {
        MessageBoxKind::Info => rfd::MessageLevel::Info,
        MessageBoxKind::Warning => rfd::MessageLevel::Warning,
        MessageBoxKind::Error => rfd::MessageLevel::Error,
        MessageBoxKind::Question => rfd::MessageLevel::Info,
    };

    let result = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(level)
        .show();

    match result {
        rfd::MessageDialogResult::Yes | rfd::MessageDialogResult::Ok => MessageBoxResult::Ok,
        _ => MessageBoxResult::Cancel,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageBoxKind {
    Info,
    Warning,
    Error,
    Question,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageBoxResult {
    Ok,
    Cancel,
}

/// Move file/folder to trash
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if the file cannot be moved to trash.
pub fn move_to_trash(path: &str) -> Result<(), ElectrobunError> {
    trash::delete(path)
        .map_err(|e| ElectrobunError::OperationFailed(format!("Failed to move to trash: {}", e)))
}

/// Show item in file manager
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if the file manager cannot be opened.
pub fn show_item_in_folder(path: &str) -> Result<(), ElectrobunError> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .status()
            .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
        if status.success() {
            return Ok(());
        }
    }
    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .status()
            .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
        if status.success() {
            return Ok(());
        }
    }
    // Fallback
    open::that(path).map_err(|e| ElectrobunError::OperationFailed(e.to_string()))
}

/// Set application menu (stub)
pub fn set_application_menu(_menu_json: &str) -> bool {
    true
}

/// Show context menu (stub)
pub fn show_context_menu(_menu_json: &str) -> bool {
    true
}
