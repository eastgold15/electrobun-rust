use electrobun_macros::eden_ipc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowParams {
    pub width: f64,
    pub height: f64,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Window {
    pub id: u32,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowError {
    pub message: String,
}

#[eden_ipc]
pub trait ElectrobunAPI {
    fn create_window(&self, params: WindowParams) -> Result<Window, WindowError>;
}
