//! Global shortcut module

use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref SHORTCUT_REGISTRY: std::sync::Mutex<HashMap<String, global_hotkey::hotkey::HotKey>> =
        std::sync::Mutex::new(HashMap::new());
    #[allow(clippy::type_complexity)]
    static ref SHORTCUT_CALLBACK: std::sync::Mutex<Option<Box<dyn Fn(&str) + Send>>> =
        std::sync::Mutex::new(None);
}

/// Set the global shortcut callback (called when any registered shortcut is pressed)
pub fn set_shortcut_callback<F>(callback: F)
where
    F: Fn(&str) + Send + 'static,
{
    let mut cb = SHORTCUT_CALLBACK.lock().unwrap_or_else(|e| e.into_inner());
    *cb = Some(Box::new(callback));
}

/// Parse shortcut string (e.g. "Ctrl+Shift+A") into a HotKey
fn parse_shortcut(shortcut: &str) -> Option<global_hotkey::hotkey::HotKey> {
    let parts: Vec<&str> = shortcut.split('+').collect();
    let mut modifiers = global_hotkey::hotkey::Modifiers::empty();
    let mut key_char: Option<String> = None;

    for part in parts {
        let part = part.trim().to_lowercase();
        match part.as_str() {
            "ctrl" | "control" => modifiers.insert(global_hotkey::hotkey::Modifiers::CONTROL),
            "alt" | "option" => modifiers.insert(global_hotkey::hotkey::Modifiers::ALT),
            "shift" => modifiers.insert(global_hotkey::hotkey::Modifiers::SHIFT),
            "meta" | "cmd" | "command" | "win" | "super" => {
                modifiers.insert(global_hotkey::hotkey::Modifiers::META)
            },
            _ => key_char = Some(part),
        }
    }

    let key_char = key_char?;
    let code = key_code_from_str(&key_char)?;

    Some(global_hotkey::hotkey::HotKey::new(Some(modifiers), unsafe { std::mem::transmute::<u8, global_hotkey::hotkey::Code>(code as u8) }))
}

/// Convert string key to winit virtual key code
fn key_code_from_str(key: &str) -> Option<u32> {
    match key {
        "a" => Some(0x1E),
        "d" => Some(0x20), "e" => Some(0x12), "f" => Some(0x21),
        "g" => Some(0x22), "h" => Some(0x23), "i" => Some(0x17),
        "j" => Some(0x24), "k" => Some(0x25), "l" => Some(0x26),
        "m" => Some(0x32), "n" => Some(0x31), "o" => Some(0x18),
        "p" => Some(0x19), "q" => Some(0x10), "r" => Some(0x13),
        "s" => Some(0x1F), "t" => Some(0x14), "u" => Some(0x16),
        "v" => Some(0x2F), "w" => Some(0x11), "x" => Some(0x2D),
        "y" => Some(0x15), "z" => Some(0x2C),
        "0" => Some(0x0B), "1" => Some(0x02), "2" => Some(0x03),
        "3" => Some(0x04), "4" => Some(0x05), "5" => Some(0x06),
        "6" => Some(0x07), "7" => Some(0x08), "8" => Some(0x09),
        "9" => Some(0x0A),
        "f1" => Some(0x3B), "f2" => Some(0x3C), "f3" => Some(0x3D),
        "f4" => Some(0x3E), "f5" => Some(0x3F), "f6" => Some(0x40),
        "f7" => Some(0x41), "f8" => Some(0x42), "f9" => Some(0x43),
        "f10" => Some(0x44), "f11" => Some(0x57), "f12" => Some(0x58),
        "escape" => Some(0x01), "space" => Some(0x39), "enter" => Some(0x1C),
        "tab" => Some(0x0F), "backspace" => Some(0x0E), "delete" => Some(0x53),
        "up" => Some(0xC8), "down" => Some(0xD0),
        "left" => Some(0xCB), "right" => Some(0xCD),
        _ => None,
    }
}

/// Register a global shortcut
pub fn register_shortcut(shortcut: &str) -> bool {
    if let Some(hotkey) = parse_shortcut(shortcut) {
        let mut registry = SHORTCUT_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(hotkey_manager) = global_hotkey::GlobalHotKeyManager::new() {
            if hotkey_manager.register(hotkey).is_ok() {
                registry.insert(shortcut.to_string(), hotkey);
                return true;
            }
        }
    }
    false
}

/// Unregister a global shortcut
pub fn unregister_shortcut(shortcut: &str) -> bool {
    let mut registry = SHORTCUT_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(hotkey) = registry.remove(shortcut) {
        if let Ok(hotkey_manager) = global_hotkey::GlobalHotKeyManager::new() {
            return hotkey_manager.unregister(hotkey).is_ok();
        }
    }
    false
}

/// Unregister all global shortcuts
pub fn unregister_all() -> bool {
    let mut registry = SHORTCUT_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    if let Ok(hotkey_manager) = global_hotkey::GlobalHotKeyManager::new() {
        for (_, hotkey) in registry.drain() {
            let _ = hotkey_manager.unregister(hotkey);
        }
        true
    } else {
        false
    }
}

/// Check if a shortcut is registered
pub fn is_shortcut_registered(shortcut: &str) -> bool {
    let registry = SHORTCUT_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    registry.contains_key(shortcut)
}
