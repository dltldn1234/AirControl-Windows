use aircontrol_core::keyboard::{KeyStroke, KeyboardSession};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

pub fn overlay_bounds(app: &AppHandle) -> Option<aircontrol_core::keyboard::KeyboardBounds> {
    let window = app.get_webview_window("air-keyboard")?;
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let bounds = aircontrol_core::keyboard::KeyboardBounds {
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
    };
    bounds.is_usable().then_some(bounds)
}

#[derive(Clone, Default)]
pub struct KeyboardController {
    session: Arc<Mutex<KeyboardSession>>,
}

impl KeyboardController {
    pub fn press(&self, key: &str) -> Result<(), String> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| "Air Keyboard is temporarily unavailable.".to_owned())?;
        let previous = *session;
        let stroke = session.press(key)?;
        if let Some(stroke) = stroke
            && let Err(error) = platform::send_keystroke(stroke)
        {
            *session = previous;
            return Err(error);
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
