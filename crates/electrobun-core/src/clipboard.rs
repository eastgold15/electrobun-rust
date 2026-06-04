//! Clipboard operations
//! Cross-platform clipboard read/write using arboard crate

use crate::error::ElectrobunError;

lazy_static::lazy_static! {
    static ref CLIPBOARD_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
}

/// Read text from clipboard
pub fn read_text() -> Result<String, ElectrobunError> {
    let _lock = CLIPBOARD_MUTEX
        .lock()
        .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
    let mut cb =
        arboard::Clipboard::new().map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
    cb.get_text()
        .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))
}

/// Write text to clipboard
pub fn write_text(text: &str) -> Result<(), ElectrobunError> {
    let _lock = CLIPBOARD_MUTEX
        .lock()
        .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
    let mut cb =
        arboard::Clipboard::new().map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
    cb.set_text(text)
        .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))
}

/// Read image from clipboard as base64 PNG
pub fn read_image_base64() -> Result<String, ElectrobunError> {
    // Simplified: returns error since image clipboard requires more complex handling
    Err(ElectrobunError::OperationFailed(
        "Image clipboard not yet implemented".to_string(),
    ))
}

/// Write image to clipboard from base64 PNG
pub fn write_image_base64(_b64_data: &str) -> Result<(), ElectrobunError> {
    // Simplified: returns error since image clipboard requires more complex handling
    Err(ElectrobunError::OperationFailed(
        "Image clipboard not yet implemented".to_string(),
    ))
}

/// Clear clipboard
pub fn clear() -> Result<(), ElectrobunError> {
    let _lock = CLIPBOARD_MUTEX
        .lock()
        .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
    let mut cb =
        arboard::Clipboard::new().map_err(|e| ElectrobunError::OperationFailed(e.to_string()))?;
    cb.clear()
        .map_err(|e| ElectrobunError::OperationFailed(e.to_string()))
}

/// Get available clipboard formats
pub fn available_formats() -> Vec<String> {
    let formats = vec!["text/plain".to_string()];
    formats
}
