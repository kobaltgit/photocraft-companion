use core_contracts::{CompanionError, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::collections::HashMap;
use tracing::info;

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    registered: HashMap<u32, String>,
}

impl HotkeyManager {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| CompanionError::Plugin(format!("Failed to initialize GlobalHotKeyManager: {e}")))?;
        Ok(Self {
            manager,
            registered: HashMap::new(),
        })
    }

    /// Parse hotkey string like "Ctrl+Shift+B"
    pub fn parse_hotkey(hotkey_str: &str) -> Option<HotKey> {
        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
        let mut modifiers = Modifiers::empty();
        let mut key_code = None;

        for part in parts {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "shift" => modifiers |= Modifiers::SHIFT,
                "alt" => modifiers |= Modifiers::ALT,
                "super" | "win" | "cmd" => modifiers |= Modifiers::SUPER,
                // Letters A-Z
                "a" => key_code = Some(Code::KeyA),
                "b" => key_code = Some(Code::KeyB),
                "c" => key_code = Some(Code::KeyC),
                "d" => key_code = Some(Code::KeyD),
                "e" => key_code = Some(Code::KeyE),
                "f" => key_code = Some(Code::KeyF),
                "g" => key_code = Some(Code::KeyG),
                "h" => key_code = Some(Code::KeyH),
                "i" => key_code = Some(Code::KeyI),
                "j" => key_code = Some(Code::KeyJ),
                "k" => key_code = Some(Code::KeyK),
                "l" => key_code = Some(Code::KeyL),
                "m" => key_code = Some(Code::KeyM),
                "n" => key_code = Some(Code::KeyN),
                "o" => key_code = Some(Code::KeyO),
                "p" => key_code = Some(Code::KeyP),
                "q" => key_code = Some(Code::KeyQ),
                "r" => key_code = Some(Code::KeyR),
                "s" => key_code = Some(Code::KeyS),
                "t" => key_code = Some(Code::KeyT),
                "u" => key_code = Some(Code::KeyU),
                "v" => key_code = Some(Code::KeyV),
                "w" => key_code = Some(Code::KeyW),
                "x" => key_code = Some(Code::KeyX),
                "y" => key_code = Some(Code::KeyY),
                "z" => key_code = Some(Code::KeyZ),
                // Digits
                "0" => key_code = Some(Code::Digit0),
                "1" => key_code = Some(Code::Digit1),
                "2" => key_code = Some(Code::Digit2),
                "3" => key_code = Some(Code::Digit3),
                "4" => key_code = Some(Code::Digit4),
                "5" => key_code = Some(Code::Digit5),
                "6" => key_code = Some(Code::Digit6),
                "7" => key_code = Some(Code::Digit7),
                "8" => key_code = Some(Code::Digit8),
                "9" => key_code = Some(Code::Digit9),
                // Function Keys
                "f1" => key_code = Some(Code::F1),
                "f2" => key_code = Some(Code::F2),
                "f3" => key_code = Some(Code::F3),
                "f4" => key_code = Some(Code::F4),
                "f5" => key_code = Some(Code::F5),
                "f6" => key_code = Some(Code::F6),
                "f7" => key_code = Some(Code::F7),
                "f8" => key_code = Some(Code::F8),
                "f9" => key_code = Some(Code::F9),
                "f10" => key_code = Some(Code::F10),
                "f11" => key_code = Some(Code::F11),
                "f12" => key_code = Some(Code::F12),
                // Special Keys
                "space" => key_code = Some(Code::Space),
                "enter" | "return" => key_code = Some(Code::Enter),
                "escape" | "esc" => key_code = Some(Code::Escape),
                "tab" => key_code = Some(Code::Tab),
                "backspace" => key_code = Some(Code::Backspace),
                "delete" | "del" => key_code = Some(Code::Delete),
                _ => {}
            }
        }

        key_code.map(|code| HotKey::new(Some(modifiers), code))
    }

    /// Register action by hotkey string
    pub fn register(&mut self, hotkey_str: &str, action_name: &str) -> Result<u32> {
        let hotkey = Self::parse_hotkey(hotkey_str)
            .ok_or_else(|| CompanionError::Plugin(format!("Unsupported hotkey format: {hotkey_str}")))?;
        let id = hotkey.id();
        self.manager
            .register(hotkey)
            .map_err(|e| CompanionError::Plugin(format!("Failed to register hotkey '{hotkey_str}': {e}")))?;
        self.registered.insert(id, action_name.to_string());
        info!("Registered global hotkey '{}' -> '{}' (id: {})", hotkey_str, action_name, id);
        Ok(id)
    }

    /// Check if any hotkey was triggered
    pub fn poll_event(&self) -> Option<String> {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == global_hotkey::HotKeyState::Released {
                return self.registered.get(&event.id).cloned();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey_valid() {
        let hk1 = HotkeyManager::parse_hotkey("Ctrl+Shift+B").unwrap();
        assert_eq!(hk1.key, Code::KeyB);
        assert_eq!(hk1.mods, Modifiers::CONTROL | Modifiers::SHIFT);

        let hk2 = HotkeyManager::parse_hotkey("alt+f4").unwrap();
        assert_eq!(hk2.key, Code::F4);
        assert_eq!(hk2.mods, Modifiers::ALT);

        let hk3 = HotkeyManager::parse_hotkey("super+space").unwrap();
        assert_eq!(hk3.key, Code::Space);
        assert_eq!(hk3.mods, Modifiers::SUPER);

        let hk4 = HotkeyManager::parse_hotkey("ctrl+alt+del").unwrap();
        assert_eq!(hk4.key, Code::Delete);
        assert_eq!(hk4.mods, Modifiers::CONTROL | Modifiers::ALT);

        let hk5 = HotkeyManager::parse_hotkey("Control + Shift + Z").unwrap();
        assert_eq!(hk5.key, Code::KeyZ);
        assert_eq!(hk5.mods, Modifiers::CONTROL | Modifiers::SHIFT);

        let hk6 = HotkeyManager::parse_hotkey("F1").unwrap();
        assert_eq!(hk6.key, Code::F1);
        assert_eq!(hk6.mods, Modifiers::empty());

        let hk7 = HotkeyManager::parse_hotkey("ctrl+5").unwrap();
        assert_eq!(hk7.key, Code::Digit5);
        assert_eq!(hk7.mods, Modifiers::CONTROL);

        let hk8 = HotkeyManager::parse_hotkey("shift+escape").unwrap();
        assert_eq!(hk8.key, Code::Escape);
        assert_eq!(hk8.mods, Modifiers::SHIFT);
    }

    #[test]
    fn test_parse_hotkey_invalid() {
        assert!(HotkeyManager::parse_hotkey("").is_none());
        assert!(HotkeyManager::parse_hotkey("Ctrl+Shift").is_none());
        assert!(HotkeyManager::parse_hotkey("UnknownKey").is_none());
        assert!(HotkeyManager::parse_hotkey("Ctrl+InvalidKey").is_none());
    }

    #[test]
    fn test_hotkey_manager() {
        if let Ok(mut mgr) = HotkeyManager::new() {
            // Test registration of an exotic combination to avoid collisions
            let res = mgr.register("Ctrl+Alt+Shift+F11", "test.action");
            assert!(res.is_ok());
            assert!(mgr.poll_event().is_none());

            // Test registration failure on invalid hotkey string
            let err_res = mgr.register("InvalidHotkeyXYZ", "test.invalid");
            assert!(err_res.is_err());
        }
    }
}


