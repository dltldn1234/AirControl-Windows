use aircontrol_core::window::{
    WindowMoveRecognizer, WindowRect, WindowSizeLimits, move_window, resize_window,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub handle: String,
    pub title: String,
    pub process_id: u32,
    pub rect: WindowRect,
    pub minimized: bool,
    pub maximized: bool,
}

#[derive(Clone, Default)]
pub struct WindowGestureController {
    state: Arc<Mutex<WindowGestureState>>,
}

#[derive(Default)]
struct WindowGestureState {
    enabled: bool,
    handle: Option<u64>,
    sensitivity: f32,
    mirrored: bool,
    move_gesture: WindowMoveRecognizer,
}

impl WindowGestureController {
    pub fn enable(&self, handle: &str, sensitivity: f32, mirrored: bool) -> Result<(), String> {
        let handle = parse_handle(handle)?;
        let target = platform::get_window(handle)?;
        if target.minimized || target.maximized {
            return Err("Restore the selected window before starting hand movement.".to_owned());
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Window control is temporarily unavailable.".to_owned())?;
        state.enabled = true;
        state.handle = Some(handle);
        state.sensitivity = if sensitivity.is_finite() {
            sensitivity.clamp(0.4, 2.0)
        } else {
            1.0
        };
        state.mirrored = mirrored;
        state.move_gesture.reset();
        Ok(())
    }

    pub fn disable(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Window control is temporarily unavailable.".to_owned())?;
        state.enabled = false;
        state.handle = None;
        state.move_gesture.reset();
        Ok(())
    }

    /// Pinch the index finger and move it to reposition the explicitly selected window.
    /// Tracking loss always ends the move gesture without applying a final delta.
    pub fn update_hand_landmarks(
        &self,
        landmarks: Option<[[f32; 2]; 21]>,
        frame_size: [u32; 2],
        pinch_sensitivity: f32,
    ) -> Result<bool, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Window control is temporarily unavailable.".to_owned())?;
        if !state.enabled {
            return Ok(false);
        }
        let Some(landmarks) = landmarks else {
            state.move_gesture.reset();
            return Ok(false);
        };
        if frame_size[0] == 0
            || frame_size[1] == 0
            || landmarks.iter().flatten().any(|value| !value.is_finite())
        {
            state.move_gesture.reset();
            return Ok(false);
        }
        let palm_size = window_point_distance(landmarks[0], landmarks[9]);
        if palm_size <= f32::EPSILON {
            state.move_gesture.reset();
            return Ok(false);
        }
        let pinch_distance = window_point_distance(landmarks[4], landmarks[8]) / palm_size;
        let mut position = [
            (landmarks[8][0] / frame_size[0] as f32).clamp(0.0, 1.0),
            (landmarks[8][1] / frame_size[1] as f32).clamp(0.0, 1.0),
        ];
        if state.mirrored {
            position[0] = 1.0 - position[0];
        }
        let update =
            state
                .move_gesture
                .update(Some(pinch_distance), Some(position), pinch_sensitivity);
        if let Some(normalized_delta) = update.normalized_delta {
            let handle = state
                .handle
                .ok_or_else(|| "No window is selected for gesture control.".to_owned())?;
            let rect = platform::get_window(handle)?.rect;
            let raw_delta = [
                normalized_delta[0] * rect.width as f32 * state.sensitivity,
                normalized_delta[1] * rect.height as f32 * state.sensitivity,
            ];
            let delta = raw_delta.map(|value| {
                if value.abs() < 1.0 {
                    0.0
                } else {
                    value.clamp(-160.0, 160.0)
                }
            });
            let target = move_window(rect, delta)
                .ok_or_else(|| "The selected window could not be moved.".to_owned())?;
            platform::set_rect(handle, target)?;
        }
        Ok(update.active)
    }
}

impl Drop for WindowGestureController {
    fn drop(&mut self) {
        if Arc::strong_count(&self.state) == 1
            && let Ok(mut state) = self.state.lock()
        {
            state.enabled = false;
            state.handle = None;
            state.move_gesture.reset();
        }
    }
}

fn window_point_distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
}

pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
    platform::list_windows()
}

pub fn foreground_window() -> Result<Option<WindowInfo>, String> {
    platform::foreground_window()
}

pub fn focus_window(handle: &str) -> Result<(), String> {
    platform::focus_window(parse_handle(handle)?)
}

pub fn move_window_by(handle: &str, delta: [f32; 2]) -> Result<WindowRect, String> {
    let handle = parse_handle(handle)?;
    let current = platform::get_window(handle)?;
    if current.minimized {
        return Err("Restore the selected window before moving it.".to_owned());
    }
    let target = move_window(current.rect, delta)
        .ok_or_else(|| "The target window or movement delta is invalid.".to_owned())?;
    platform::set_rect(handle, target)?;
    Ok(target)
}

pub fn resize_window_by(handle: &str, scale: f32) -> Result<WindowRect, String> {
    let handle = parse_handle(handle)?;
    let current = platform::get_window(handle)?;
    if current.minimized {
        return Err("Restore the selected window before resizing it.".to_owned());
    }
    let limits = WindowSizeLimits {
        minimum: [320, 200],
        maximum: [8192, 8192],
    };
    let target = resize_window(current.rect, scale, limits)
        .ok_or_else(|| "The target window or size limits are invalid.".to_owned())?;
    platform::set_rect(handle, target)?;
    Ok(platform::get_window(handle)?.rect)
}

pub fn minimize_window(handle: &str) -> Result<(), String> {
    platform::show_window(parse_handle(handle)?, platform::ShowAction::Minimize)
}

pub fn maximize_window(handle: &str) -> Result<(), String> {
    platform::show_window(parse_handle(handle)?, platform::ShowAction::Maximize)
}

fn parse_handle(handle: &str) -> Result<u64, String> {
    u64::from_str_radix(handle, 16).map_err(|_| "The selected window handle is invalid.".to_owned())
}

#[cfg(test)]
mod tests {
    use super::parse_handle;

    #[test]
    fn native_window_handles_round_trip_as_hex_strings() {
        assert_eq!(parse_handle("7ffde123"), Ok(0x7ffde123));
        assert!(parse_handle("not-a-handle").is_err());
        assert!(parse_handle("").is_err());
    }
}

#[cfg(windows)]
mod platform {
    use super::WindowInfo;
    use aircontrol_core::window::{WindowCandidate, WindowRect, is_supported_window};
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
    use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GW_OWNER, GWL_EXSTYLE, GetForegroundWindow, GetWindow, GetWindowLongPtrW,
        GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow,
        IsWindowVisible, IsZoomed, SW_MINIMIZE, SW_RESTORE, SW_SHOWMAXIMIZED, SWP_NOACTIVATE,
        SWP_NOZORDER, SetForegroundWindow, SetWindowPos, ShowWindow, WNDENUMPROC, WS_EX_TOOLWINDOW,
    };

    const TITLE_CAPACITY: usize = 512;
    const MINIMUM_WIDTH: u32 = 160;
    const MINIMUM_HEIGHT: u32 = 100;

    #[derive(Clone, Copy)]
    pub enum ShowAction {
        Minimize,
        Maximize,
    }

    pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
        let mut windows = Vec::<WindowInfo>::new();
        let callback_context = (&mut windows as *mut Vec<WindowInfo>) as LPARAM;
        let success = unsafe { EnumWindows(Some(collect_window), callback_context) };
        if success == 0 {
            return Err("Windows could not enumerate application windows.".to_owned());
        }
        Ok(windows)
    }

    pub fn foreground_window() -> Result<Option<WindowInfo>, String> {
        let handle = unsafe { GetForegroundWindow() };
        if handle.is_null() {
            return Ok(None);
        }
        Ok(inspect_window(handle))
    }

    pub fn focus_window(handle: u64) -> Result<(), String> {
        let hwnd = validated_handle(handle)?;
        if unsafe { IsIconic(hwnd) } != 0 {
            unsafe { ShowWindow(hwnd, SW_RESTORE) };
        }
        if unsafe { SetForegroundWindow(hwnd) } == 0 {
            return Err(
                "Windows declined to activate this window under its foreground restrictions."
                    .to_owned(),
            );
        }
        Ok(())
    }

    pub fn get_window(handle: u64) -> Result<WindowInfo, String> {
        inspect_window(validated_handle(handle)?)
            .ok_or_else(|| "The selected window is no longer supported or available.".to_owned())
    }

    pub fn set_rect(handle: u64, rect: WindowRect) -> Result<(), String> {
        let hwnd = validated_handle(handle)?;
        if rect.width < MINIMUM_WIDTH || rect.height < MINIMUM_HEIGHT {
            return Err("The requested window size is below the safe minimum.".to_owned());
        }
        let width = i32::try_from(rect.width)
            .map_err(|_| "The requested window width is too large.".to_owned())?;
        let height = i32::try_from(rect.height)
            .map_err(|_| "The requested window height is too large.".to_owned())?;
        let success = unsafe {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                rect.x,
                rect.y,
                width,
                height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
        };
        if success == 0 {
            return Err("Windows did not allow the selected window to be changed.".to_owned());
        }
        Ok(())
    }

    pub fn show_window(handle: u64, action: ShowAction) -> Result<(), String> {
        let hwnd = validated_handle(handle)?;
        let command = match action {
            ShowAction::Minimize => SW_MINIMIZE,
            ShowAction::Maximize => SW_SHOWMAXIMIZED,
        };
        unsafe { ShowWindow(hwnd, command) };
        Ok(())
    }

    fn validated_handle(handle: u64) -> Result<HWND, String> {
        let hwnd = usize::try_from(handle)
            .ok()
            .map(|value| value as HWND)
            .filter(|value| !value.is_null())
            .ok_or_else(|| "The selected window handle is invalid.".to_owned())?;
        inspect_window(hwnd)
            .map(|_| hwnd)
            .ok_or_else(|| "The selected window is no longer supported or available.".to_owned())
    }

    unsafe extern "system" fn collect_window(hwnd: HWND, context: LPARAM) -> i32 {
        let windows = unsafe { &mut *(context as *mut Vec<WindowInfo>) };
        if let Some(window) = inspect_window(hwnd) {
            windows.push(window);
        }
        1
    }

    fn inspect_window(hwnd: HWND) -> Option<WindowInfo> {
        if hwnd.is_null() || unsafe { IsWindow(hwnd) } == 0 || unsafe { IsWindowVisible(hwnd) } == 0
        {
            return None;
        }

        let mut title = [0u16; TITLE_CAPACITY];
        let title_length =
            unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), TITLE_CAPACITY as i32) };
        if title_length <= 0 {
            return None;
        }
        let title = String::from_utf16_lossy(&title[..title_length as usize]);
        let mut process_id = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
        let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
        let mut cloaked = 0u32;
        let cloak_query = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED as u32,
                (&mut cloaked as *mut u32).cast(),
                size_of::<u32>() as u32,
            )
        };
        let candidate = WindowCandidate {
            handle: hwnd as usize as u64,
            title: &title,
            process_id,
            visible: true,
            owned: !unsafe { GetWindow(hwnd, GW_OWNER) }.is_null(),
            cloaked: cloak_query < 0 || cloaked != 0,
            tool_window: ex_style & WS_EX_TOOLWINDOW != 0,
        };
        if !is_supported_window(candidate, std::process::id()) {
            return None;
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
            return None;
        }
        let width = rect.right.checked_sub(rect.left)?;
        let height = rect.bottom.checked_sub(rect.top)?;
        if width <= 0 || height <= 0 {
            return None;
        }
        Some(WindowInfo {
            handle: format!("{:x}", candidate.handle),
            title,
            process_id,
            rect: WindowRect {
                x: rect.left,
                y: rect.top,
                width: width as u32,
                height: height as u32,
            },
            minimized: unsafe { IsIconic(hwnd) } != 0,
            maximized: unsafe { IsZoomed(hwnd) } != 0,
        })
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{WindowInfo, WindowRect};

    #[derive(Clone, Copy)]
    pub enum ShowAction {
        Minimize,
        Maximize,
    }

    const ERROR: &str = "Window control is available only in the Windows desktop app.";

    pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
        Err(ERROR.to_owned())
    }

    pub fn foreground_window() -> Result<Option<WindowInfo>, String> {
        Err(ERROR.to_owned())
    }

    pub fn focus_window(_handle: u64) -> Result<(), String> {
        Err(ERROR.to_owned())
    }

    pub fn get_window(_handle: u64) -> Result<WindowInfo, String> {
        Err(ERROR.to_owned())
    }

    pub fn set_rect(_handle: u64, _rect: WindowRect) -> Result<(), String> {
        Err(ERROR.to_owned())
    }

    pub fn show_window(_handle: u64, _action: ShowAction) -> Result<(), String> {
        Err(ERROR.to_owned())
    }
}
