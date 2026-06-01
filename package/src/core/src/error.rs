//! Error types for Electrobun Core

use thiserror::Error;

/// Electrobun error types
#[derive(Error, Debug)]
pub enum ElectrobunError {
    #[error("Window not found: {0}")]
    WindowNotFound(u32),
    
    #[error("Webview not found: {0}")]
    WebviewNotFound(u32),
    
    #[error("WgpuView not found: {0}")]
    WgpuViewNotFound(u32),
    
    #[error("Tray not found: {0}")]
    TrayNotFound(u32),
    
    #[error("Window operation failed: {0}")]
    WindowOperationFailed(String),
    
    #[error("Webview operation failed: {0}")]
    WebviewOperationFailed(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    
    #[error("FFI error: {0}")]
    FfiError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl From<std::io::Error> for ElectrobunError {
    fn from(err: std::io::Error) -> Self {
        ElectrobunError::WindowOperationFailed(err.to_string())
    }
}

impl From<serde_json::Error> for ElectrobunError {
    fn from(err: serde_json::Error) -> Self {
        ElectrobunError::InitializationFailed(err.to_string())
    }
}
