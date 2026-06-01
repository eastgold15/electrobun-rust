//! Global keyboard shortcuts
//! Uses global-hotkey crate for cross-platform support

use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref SHORTCUT_REGISTRY: std::sync::Mutex<HashMap<String, global_hotkey::hotkey::HotKey>> =
        std::sync::Mutex::new(HashMap::new());
    static ref SHORTCUT_CALLBACK: std::sync::Mutex<Option<Box<dyn Fn(&str) + Send>>> =
        std::sync::Mutex::new(None);
}

/// Set the global shortcut callback (called when any registered shortcut is pressed)
pub fn set_shortcut_callback<F>(callback: F)
where
    F: Fn(&str) + Send + 'static,
{
    let mut cb = SHORTCUT_CALLBACK.lock().unwrap();
    *cb = Some(Box::new(callback));
}

/// Parse a shortcut string like "Ctrl+Shift+A" into a hotkey
fn parse_shortcut(shortcut: &str) -> Option<global_hotkey::hotkey::HotKey> {
    let parts: Vec<&str> = shortcut.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = global_hotkey::hotkey::Modifiers::empty();
    let mut key_char = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.insert(global_hotkey::hotkey::Modifiers::CONTROL),
            "alt" | "option" => modifiers.insert(global_hotkey::hotkey::Modifiers::ALT),
            "shift" => modifiers.insert(global_hotkey::hotkey::Modifiers::SHIFT),
            "meta" | "cmd" | "command" | "win" | "super" => modifiers.insert(global_hotkey::hotkey::Modifiers::META),
            _ => key_char = Some(part),
        }
    }

    let code = match key_char?.to_lowercase().as_str() {
        "a" => global_hotkey::hotkey::Code::KeyA,
        "b" => global_hotkey::hotkey::Code::KeyB,
        "c" => global_hotkey::hotkey::Code::KeyC,
        "d" => global_hotkey::hotkey::Code::KeyD,
        "e" => global_hotkey::hotkey::Code::KeyE,
        "f" => global_hotkey::hotkey::Code::KeyF,
        "g" => global_hotkey::hotkey::Code::KeyG,
        "h" => global_hotkey::hotkey::Code::KeyH,
        "i" => global_hotkey::hotkey::Code::KeyI,
        "j" => global_hotkey::hotkey::Code::KeyJ,
        "k" => global_hotkey::hotkey::Code::KeyK,
        "l" => global_hotkey::hotkey::Code::KeyL,
        "m" => global_hotkey::hotkey::Code::KeyM,
        "n" => global_hotkey::hotkey::Code::KeyN,
        "o" => global_hotkey::hotkey::Code::KeyO,
        "p" => global_hotkey::hotkey::Code::KeyP,
        "q" => global_hotkey::hotkey::Code::KeyQ,
        "r" => global_hotkey::hotkey::Code::KeyR,
        "s" => global_hotkey::hotkey::Code::KeyS,
        "t" => global_hotkey::hotkey::Code::KeyT,
        "u" => global_hotkey::hotkey::Code::KeyU,
        "v" => global_hotkey::hotkey::Code::KeyV,
        "w" => global_hotkey::hotkey::Code::KeyW,
        "x" => global_hotkey::hotkey::Code::KeyX,
        "y" => global_hotkey::hotkey::Code::KeyY,
        "z" => global_hotkey::hotkey::Code::KeyZ,
        "0" => global_hotkey::hotkey::Code::Digit0,
        "1" => global_hotkey::hotkey::Code::Digit1,
        "2" => global_hotkey::hotkey::Code::Digit2,
        "3" => global_hotkey::hotkey::Code::Digit3,
        "4" => global_hotkey::hotkey::Code::Digit4,
        "5" => global_hotkey::hotkey::Code::Digit5,
        "6" => global_hotkey::hotkey::Code::Digit6,
        "7" => global_hotkey::hotkey::Code::Digit7,
        "8" => global_hotkey::hotkey::Code::Digit8,
        "9" => global_hotkey::hotkey::Code::Digit9,
        "f1" => global_hotkey::hotkey::Code::F1,
        "f2" => global_hotkey::hotkey::Code::F2,
        "f3" => global_hotkey::hotkey::Code::F3,
        "f4" => global_hotkey::hotkey::Code::F4,
        "f5" => global_hotkey::hotkey::Code::F5,
        "f6" => global_hotkey::hotkey::Code::F6,
        "f7" => global_hotkey::hotkey::Code::F7,
        "f8" => global_hotkey::hotkey::Code::F8,
        "f9" => global_hotkey::hotkey::Code::F9,
        "f10" => global_hotkey::hotkey::Code::F10,
        "f11" => global_hotkey::hotkey::Code::F11,
        "f12" => global_hotkey::hotkey::Code::F12,
        "space" => global_hotkey::hotkey::Code::Space,
        "enter" | "return" => global_hotkey::hotkey::Code::Enter,
        "escape" | "esc" => global_hotkey::hotkey::Code::Escape,
        "tab" => global_hotkey::hotkey::Code::Tab,
        "delete" | "del" => global_hotkey::hotkey::Code::Delete,
        "backspace" => global_hotkey::hotkey::Code::Backspace,
        "up" => global_hotkey::hotkey::Code::ArrowUp,
        "down" => global_hotkey::hotkey::Code::ArrowDown,
        "left" => global_hotkey::hotkey::Code::ArrowLeft,
        "right" => global_hotkey::hotkey::Code::ArrowRight,
        _ => return None,
    };

    Some(global_hotkey::hotkey::HotKey::new(Some(modifiers), code))
}

/// Register a global shortcut
pub fn register_shortcut(shortcut: &str) -> bool {
    if let Some(hotkey) = parse_shortcut(shortcut) {
        let mut registry = SHORTCUT_REGISTRY.lock().unwrap();
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
    let mut registry = SHORTCUT_REGISTRY.lock().unwrap();
    if let Some(hotkey) = registry.remove(shortcut) {
        if let Ok(hotkey_manager) = global_hotkey::GlobalHotKeyManager::new() {
            return hotkey_manager.unregister(hotkey).is_ok();
        }
    }
    false
}

/// Unregister all global shortcuts
pub fn unregister_all() -> bool {
    let mut registry = SHORTCUT_REGISTRY.lock().unwrap();
    if let Ok(hotkey_manager) = global_hotkey::GlobalHotKeyManager::new() {
        for (_, hotkey) in registry.drain() {
            let _ = hotkey_manager.unregister(hotkey);
        }
        return true;
    }
    registry.clear();
    false
}

/// Check if a shortcut is registered
pub fn is_shortcut_registered(shortcut: &str) -> bool {
    let registry = SHORTCUT_REGISTRY.lock().unwrap();
    registry.contains_key(shortcut)
}
