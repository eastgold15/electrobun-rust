//! System dialogs and native menus
//! Uses rfd (Rusty File Dialogs) for file dialogs and message boxes

use crate::error::ElectrobunError;

/// Open a file dialog
pub fn open_file_dialog(
    title: &str,
    default_path: &str,
    filters: &[(String, Vec<String>)],  // (description, extensions)
    multi: bool,
) -> Result<Vec<String>, ElectrobunError> {
    let mut dialog = rfd::FileDialog::new()
        .set_title(title);

    if !default_path.is_empty() {
        dialog = dialog.set_directory(default_path);
    }

    for (desc, exts) in filters {
        let ext_refs: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(desc, &ext_refs);
    }

    let result = if multi {
        dialog.pick_multiple_files()
    } else {
        dialog.pick_file().map(|f| vec![f.to_string_lossy().to_string()])
    };

    Ok(result.unwrap_or_default())
}

/// Save file dialog
pub fn save_file_dialog(
    title: &str,
    default_name: &str,
    default_path: &str,
) -> Result<Option<String>, ElectrobunError> {
    let mut dialog = rfd::FileDialog::new()
        .set_title(title);

    if !default_name.is_empty() {
        dialog = dialog.set_file_name(default_name);
    }
    if !default_path.is_empty() {
        dialog = dialog.set_directory(default_path);
    }

    Ok(dialog.save_file().map(|p| p.to_string_lossy().to_string()))
}

/// Show a message box
pub fn show_message_box(
    title: &str,
    message: &str,
    kind: MessageBoxKind,
) -> MessageBoxResult {
    let dialog = rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message);

    let dialog = match kind {
        MessageBoxKind::Info => dialog.set_level(rfd::MessageLevel::Info),
        MessageBoxKind::Warning => dialog.set_level(rfd::MessageLevel::Warning),
        MessageBoxKind::Error => dialog.set_level(rfd::MessageLevel::Error),
        MessageBoxKind::Question => dialog.set_level(rfd::MessageLevel::Info),
    };

    let dialog = dialog.set_buttons(rfd::Buttons::OkCancel);
    let result = dialog.show();

    match result {
        rfd::MessageDialogResult::Yes | rfd::MessageDialogResult::Ok => MessageBoxResult::Ok,
        rfd::MessageDialogResult::No | rfd::MessageDialogResult::Cancel => MessageBoxResult::Cancel,
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
pub fn move_to_trash(path: &str) -> Result<(), ElectrobunError> {
    trash::delete(path)
        .map_err(|e| ElectrobunError::OperationFailed(format!("Failed to move to trash: {}", e)))
}

/// Show item in file manager
pub fn show_item_in_folder(path: &str) -> Result<(), ElectrobunError> {
    opener::open(path)
        .map_err(|e| ElectrobunError::OperationFailed(format!("Failed to show item: {}", e)))
}

/// Set application menu (stub - would need native menu implementation)
pub fn set_application_menu(_menu_json: &str) -> bool {
    // TODO: Implement native menu on macOS/Windows/Linux
    true
}

/// Show context menu (stub)
pub fn show_context_menu(_menu_json: &str) -> bool {
    // TODO: Implement native context menu
    true
}
