//! Transport module - WebSocket communication

use crate::error::ElectrobunError;
use crate::types::PendingHostMessage;
use crate::webview::PENDING_HOST_MESSAGES;
use std::sync::{Arc, Mutex};
use std::os::raw::c_int;

/// WebSocket constants
pub const WEBSOCKET_MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
pub const WEBSOCKET_PAYLOAD_LIMIT: usize = 1024 * 1024 * 500; // 500MB
pub const WEBSOCKET_PORT_RANGE_START: u16 = 50000;
pub const WEBSOCKET_PORT_RANGE_END: u16 = 65535;

/// Host transport state
lazy_static::lazy_static! {
    pub static ref HOST_TRANSPORT_STATE: Arc<Mutex<super::types::HostTransportState>> = 
        Arc::new(Mutex::new(super::types::HostTransportState::default()));
    
    pub static ref HOST_MESSAGE_WAKEUP_FD: Arc<Mutex<Option<(c_int, c_int)>>> = 
        Arc::new(Mutex::new(None));
}

/// Get wakeup file descriptor
pub fn get_wakeup_fd() -> Option<c_int> {
    let fds = HOST_MESSAGE_WAKEUP_FD.lock().unwrap_or_else(|e| e.into_inner());
    fds.map(|(read_fd, _)| read_fd)
}

/// Enqueue a pending host message
pub fn enqueue_pending_host_message(webview_id: u32, message: String) {
    let pending = PendingHostMessage {
        webview_id,
        message,
    };
    
    let mut queue = PENDING_HOST_MESSAGES.lock().unwrap_or_else(|e| e.into_inner());
    queue.push_back(pending);
    
    // Signal the wakeup fd
    signal_wakeup();
}

/// Pop the next queued message
pub fn pop_next_message() -> Option<(u32, String)> {
    let mut queue = PENDING_HOST_MESSAGES.lock().unwrap_or_else(|e| e.into_inner());
    
    if let Some(msg) = queue.pop_front() {
        Some((msg.webview_id, msg.message))
    } else {
        None
    }
}

/// Signal the wakeup fd
fn signal_wakeup() {
    if let Ok(mut fds) = HOST_MESSAGE_WAKEUP_FD.lock() {
        if let Some((_, write_fd)) = *fds {
            // Write a single byte to signal
            let byte = [1u8];
            unsafe {
                libc::write(write_fd, byte.as_ptr() as *const libc::c_void, 1);
            }
        }
    }
}

/// Send message to webview
pub fn send_message_to_webview(webview_id: u32, message: &str) -> bool {
    // Queue the message for the webview
    enqueue_pending_host_message(webview_id, message.to_string());
    true
}

/// Decode base64
pub fn decode_base64(input: &[u8]) -> Result<Vec<u8>, ElectrobunError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| ElectrobunError::TransportError(e.to_string()))
}

/// Encode base64
pub fn encode_base64(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input)
}

/// Parse WebSocket frame (simplified)
pub struct WebSocketFrame {
    pub opcode: u8,
    pub payload: Vec<u8>,
    pub fin: bool,
}

/// Parse a WebSocket frame from bytes
pub fn parse_websocket_frame(data: &[u8]) -> Result<WebSocketFrame, ElectrobunError> {
    if data.len() < 2 {
        return Err(ElectrobunError::TransportError("Frame too short".into()));
    }
    
    let first_byte = data[0];
    let second_byte = data[1];
    
    let fin = (first_byte & 0x80) != 0;
    let opcode = first_byte & 0x0F;
    let masked = (second_byte & 0x80) != 0;
    
    let mut payload_len = (second_byte & 0x7F) as usize;
    
    let mut offset = 2;
    
    // Extended payload length
    if payload_len == 126 {
        if data.len() < 4 {
            return Err(ElectrobunError::TransportError("Frame too short for extended length".into()));
        }
        payload_len = ((data[2] as usize) << 8) | (data[3] as usize);
        offset = 4;
    } else if payload_len == 127 {
        if data.len() < 10 {
            return Err(ElectrobunError::TransportError("Frame too short for extended length".into()));
        }
        payload_len = 0;
        for i in 0..8 {
            payload_len = (payload_len << 8) | (data[2 + i] as usize);
        }
        offset = 10;
    }
    
    // Masking key
    let mask = if masked {
        if data.len() < offset + 4 {
            return Err(ElectrobunError::TransportError("Frame too short for mask".into()));
        }
        let key = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        offset += 4;
        Some(key)
    } else {
        None
    };
    
    // Get payload
    if data.len() < offset + payload_len {
        return Err(ElectrobunError::TransportError("Frame payload incomplete".into()));
    }
    
    let mut payload = data[offset..offset + payload_len].to_vec();
    
    // Unmask if needed
    if let Some([m0, m1, m2, m3]) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= [m0, m1, m2, m3][i % 4];
        }
    }
    
    Ok(WebSocketFrame {
        opcode,
        payload,
        fin,
    })
}

/// Build WebSocket frame
pub fn build_websocket_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    
    // First byte: FIN + opcode
    frame.push(0x80 | (opcode & 0x0F));
    
    // Second byte: payload length (no masking for server->client)
    let len = payload.len();
    if len < 126 {
        frame.push(len as u8);
    } else if len < 65536 {
        frame.push(126);
        frame.push((len >> 8) as u8);
        frame.push((len & 0xFF) as u8);
    } else {
        frame.push(127);
        for i in (0..8).rev() {
            frame.push((len >> (i * 8)) as u8);
        }
    }
    
    // Payload
    frame.extend_from_slice(payload);
    
    frame
}

/// WebSocket opcodes
pub const WS_OPCODE_CONTINUATION: u8 = 0x0;
pub const WS_OPCODE_TEXT: u8 = 0x1;
pub const WS_OPCODE_BINARY: u8 = 0x2;
pub const WS_OPCODE_CLOSE: u8 = 0x8;
pub const WS_OPCODE_PING: u8 = 0x9;
pub const WS_OPCODE_PONG: u8 = 0xA;
