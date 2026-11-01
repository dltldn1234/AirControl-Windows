use aircontrol_core::keyboard::{KeyStroke, KeyboardSession};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct KeyboardController {
    session: Arc<Mutex<KeyboardSession>>,
}

impl KeyboardController {
    pub fn press(&self, key: &str) -> Result<(), String> {
        let stroke = self
            .session
            .lock()
            .map_err(|_| "Air Keyboard is temporarily unavailable.".to_owned())?
            .press(key)?;
        if let Some(stroke) = stroke {
            platform::send_keystroke(stroke)?;
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<(), String> {
        self.session
            .lock()
            .map_err(|_| "Air Keyboard is temporarily unavailable.".to_owned())?
            .reset();
        Ok(())
    }

    pub fn reset_modifiers(&self) -> Result<(), String> {
        self.session
            .lock()
            .map_err(|_| "Air Keyboard is temporarily unavailable.".to_owned())?
            .reset_modifiers();
        Ok(())
    }
}

#[cfg(windows)]
mod platform {
    use super::KeyStroke;
    use aircontrol_core::keyboard::VK_SHIFT;
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    };

    pub fn send_keystroke(stroke: KeyStroke) -> Result<(), String> {
        let mut events = Vec::with_capacity(if stroke.shift { 4 } else { 2 });
        if stroke.shift {
            events.push(key_event(VK_SHIFT, false));
        }
        events.push(key_event(stroke.virtual_key, false));
        events.push(key_event(stroke.virtual_key, true));
        if stroke.shift {
            events.push(key_event(VK_SHIFT, true));
        }
        let sent = unsafe {
            SendInput(
                events.len() as u32,
                events.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        };
        if sent == events.len() as u32 {
            Ok(())
        } else {
            // A partial SendInput batch could have left Shift or the key down.
            // Best-effort key-up events make failure recovery safe.
            let release = [
                key_event(stroke.virtual_key, true),
                key_event(VK_SHIFT, true),
            ];
            let _ = unsafe {
                SendInput(
                    release.len() as u32,
                    release.as_ptr(),
                    size_of::<INPUT>() as i32,
                )
            };
            Err(
                "Windows rejected a keyboard event. Higher-integrity applications may block input."
                    .to_owned(),
            )
        }
    }

    fn key_event(virtual_key: u16, key_up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: virtual_key,
                    wScan: 0,
                    dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::KeyStroke;
    pub fn send_keystroke(_stroke: KeyStroke) -> Result<(), String> {
        Err("System keyboard input is available only in the Windows desktop app.".to_owned())
    }
}
