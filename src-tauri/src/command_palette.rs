use aircontrol_core::RuntimeSnapshot;
use aircontrol_core::command_palette::PaletteCommand;
use std::sync::Mutex;

pub fn execute_if_active(
    runtime: &Mutex<RuntimeSnapshot>,
    command: PaletteCommand,
) -> Result<(), String> {
    let snapshot = runtime
        .lock()
        .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?;
    let validated = aircontrol_core::command_palette::command_for_active_palette(
        snapshot.control_mode,
        command.id(),
    )?;
    let result = execute(validated);
    drop(snapshot);
    result
}

pub fn execute_by_id(runtime: &Mutex<RuntimeSnapshot>, command_id: &str) -> Result<(), String> {
    let snapshot = runtime
        .lock()
        .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?;
    let command = aircontrol_core::command_palette::command_for_active_palette(
        snapshot.control_mode,
        command_id,
    )?;
    let result = execute(command);
    drop(snapshot);
    result
}

pub fn execute(command: PaletteCommand) -> Result<(), String> {
    match command {
        PaletteCommand::NextApplication => crate::window_manager::switch_to_next_application(),
        PaletteCommand::SnapWindowLeft => {
            crate::window_manager::focus_next_application_window()?;
            platform::send_shortcut(&[0x5b, 0x25])
        }
        PaletteCommand::SnapWindowRight => {
            crate::window_manager::focus_next_application_window()?;
            platform::send_shortcut(&[0x5b, 0x27])
        }
        PaletteCommand::PlayPause => platform::send_key(0xb3),
        PaletteCommand::VolumeDown => platform::send_key(0xae),
        PaletteCommand::VolumeUp => platform::send_key(0xaf),
    }
}

#[cfg(windows)]
mod platform {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    };

    pub fn send_key(virtual_key: u16) -> Result<(), String> {
        send_sequence(&[virtual_key])
    }

    pub fn send_shortcut(virtual_keys: &[u16]) -> Result<(), String> {
        send_sequence(virtual_keys)
    }

    fn send_sequence(virtual_keys: &[u16]) -> Result<(), String> {
        let mut events = Vec::with_capacity(virtual_keys.len() * 2);
        events.extend(virtual_keys.iter().map(|key| key_event(*key, false)));
        events.extend(virtual_keys.iter().rev().map(|key| key_event(*key, true)));
        let sent = unsafe {
            SendInput(
                events.len() as u32,
                events.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        };
        if sent > 0 {
            crate::input_diagnostics::record_input_events(sent as usize);
        }
        if sent == events.len() as u32 {
            Ok(())
        } else {
            let releases: Vec<_> = virtual_keys
                .iter()
                .map(|key| key_event(*key, true))
                .collect();
            let released = unsafe {
                SendInput(
                    releases.len() as u32,
                    releases.as_ptr(),
                    size_of::<INPUT>() as i32,
                )
            };
            if released > 0 {
                crate::input_diagnostics::record_input_events(released as usize);
            }
            Err("Windows rejected the command input. A protected or higher-integrity app may block it.".to_owned())
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
    pub fn send_key(_virtual_key: u16) -> Result<(), String> {
        Err("Command palette system input is only available in the Windows desktop app.".to_owned())
    }

    pub fn send_shortcut(_virtual_keys: &[u16]) -> Result<(), String> {
        Err("Command palette system input is only available in the Windows desktop app.".to_owned())
    }
}
