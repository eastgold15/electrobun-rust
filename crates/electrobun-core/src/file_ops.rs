// File and URL operations

use crate::error::ElectrobunError;

/// Open a URL in the default browser
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if no default browser is available or the URL is invalid.
pub fn open_external(url: &str) -> Result<(), ElectrobunError> {
    open::that(url)
        .map_err(|e| ElectrobunError::OperationFailed(format!("Failed to open URL: {}", e)))
}

/// Open a file path in the default application
///
/// # Errors
///
/// Returns [`ElectrobunError::OperationFailed`] if no default application is registered for the path.
pub fn open_path(path: &str) -> Result<(), ElectrobunError> {
    open::that(path)
        .map_err(|e| ElectrobunError::OperationFailed(format!("Failed to open path: {}", e)))
}
