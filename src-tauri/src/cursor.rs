use aircontrol_core::cursor::{
    VirtualDesktop, apply_cursor_sensitivity, map_normalized_cursor, normalize_absolute_cursor,
};
use aircontrol_core::filter::OneEuroCursorFilter;
use aircontrol_core::gesture::{
    PinchAction, PinchRecognizer, PinchSource, RightClickRecognizer,
    choose_pinch_source_with_enabled, two_finger_scroll_position,
};
use aircontrol_core::scroll::ScrollRecognizer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Default)]
pub struct CursorController {
    state: Arc<Mutex<CursorState>>,
}

#[derive(Default)]
struct CursorState {
    enabled: bool,
    mirrored: bool,
    sensitivity: f32,
    smoothing: f32,
    left_button_down: bool,
    right_button_down: bool,
    pinch: PinchRecognizer,
    right_pinch: RightClickRecognizer,
    active_pinch: Option<PinchSource>,
    scroll: ScrollRecognizer,
    scroll_sensitivity: f32,
    reverse_scroll: bool,
    right_click_enabled: bool,
    filter: OneEuroCursorFilter,
}

impl CursorController {
    pub fn enable(
        &self,
        mirrored: bool,
        sensitivity: f32,
        smoothing: f32,
        scroll_sensitivity: f32,
        reverse_scroll: bool,
        right_click_enabled: bool,
    ) -> Result<(), String> {
        let _ = current_virtual_desktop()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Cursor control is temporarily unavailable.".to_owned())?;
        state.mirrored = mirrored;
        state.sensitivity = if sensitivity.is_finite() {
            sensitivity.clamp(0.5, 2.0)
        } else {
            1.0
        };
        state.smoothing = if smoothing.is_finite() {
            smoothing.clamp(0.0, 0.9)
        } else {
            0.4
        };
        state.scroll_sensitivity = if scroll_sensitivity.is_finite() {
            scroll_sensitivity.clamp(0.4, 3.0)
        } else {
            1.0
        };
        state.reverse_scroll = reverse_scroll;
        state.right_click_enabled = right_click_enabled;
        let _ = state.pinch.reset();
        state.right_pinch.reset();
        state.filter.reset();
        state.scroll.reset();
        state.active_pinch = None;
        state.enabled = true;
        Ok(())
    }

    pub fn disable(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Cursor control is temporarily unavailable.".to_owned())?;
        state.enabled = false;
        let _ = state.pinch.reset();
        state.right_pinch.reset();
        state.active_pinch = None;
        state.scroll.reset();
        state.filter.reset();
        let left_result = release_left_button(&mut state);
        let right_result = release_right_button(&mut state);
        left_result.and(right_result)
    }

    /// Move from the index fingertip's normalized camera position. Missing
    /// tracking input stops movement and releases any held button.
    pub fn update_hand_landmarks(
        &self,
        landmarks: Option<[[f32; 2]; 21]>,
        frame_size: [u32; 2],
        pinch_sensitivity: f32,
    ) -> Result<PinchAction, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Cursor control is temporarily unavailable.".to_owned())?;
        if !state.enabled {
            return Ok(PinchAction::None);
        }
        let Some(landmarks) = landmarks else {
            return release_for_tracking_loss(&mut state);
        };
        let Some(point) = normalized_landmark(landmarks[8], frame_size) else {
            return release_for_tracking_loss(&mut state);
        };

        let action = update_gestures(&mut state, &landmarks, frame_size, pinch_sensitivity)?;
        if action == PinchAction::Scroll {
            return Ok(action);
        }

        let desktop = current_virtual_desktop()?;
        let smoothing = state.smoothing;
        let Some(filtered_point) = state.filter.update(point, smoothing, Instant::now()) else {
            return release_for_tracking_loss(&mut state);
        };
        let Some(sensitive_point) = apply_cursor_sensitivity(filtered_point, state.sensitivity)
        else {
            return release_for_tracking_loss(&mut state);
        };
        let Some(screen_point) = map_normalized_cursor(sensitive_point, desktop, state.mirrored)
        else {
            return release_for_tracking_loss(&mut state);
        };
        let absolute = normalize_absolute_cursor(screen_point, desktop)
            .ok_or_else(|| "The Windows virtual desktop has no usable area.".to_owned())?;
        platform::move_cursor(absolute)?;
        Ok(action)
    }
}

fn release_for_tracking_loss(state: &mut CursorState) -> Result<PinchAction, String> {
    let was_scrolling = state.scroll.is_active();
    state.filter.reset();
    let action = state.pinch.reset();
    state.right_pinch.reset();
    state.active_pinch = None;
    state.scroll.reset();
    let dispatch_result = dispatch_gesture_action(action, state);
    let left_result = release_left_button(state);
    let right_result = release_right_button(state);
    dispatch_result.and(left_result).and(right_result)?;
    if was_scrolling && action == PinchAction::None {
        Ok(PinchAction::Cursor)
    } else {
        Ok(action)
    }
}

fn update_gestures(
    state: &mut CursorState,
    landmarks: &[[f32; 2]; 21],
    frame_size: [u32; 2],
    pinch_sensitivity: f32,
) -> Result<PinchAction, String> {
    let left_distance = normalized_pinch_distance(landmarks, 8);
    let right_distance = normalized_pinch_distance(landmarks, 12);
    let source = state.active_pinch.or_else(|| {
        choose_pinch_source_with_enabled(
            left_distance,
            right_distance,
            pinch_sensitivity,
            state.right_click_enabled,
        )
    });

    if let Some(source) = source {
        state.scroll.reset();
        match source {
            PinchSource::LeftClick => {
                state.right_pinch.reset();
                let action = state
                    .pinch
                    .update(left_distance, pinch_sensitivity, Instant::now());
                dispatch_gesture_action(action, state)?;
                state.active_pinch = state.pinch.is_active().then_some(PinchSource::LeftClick);
                return Ok(action);
            }
            PinchSource::RightClick => {
                let _ = state.pinch.reset();
                let clicked =
                    state
                        .right_pinch
                        .update(right_distance, pinch_sensitivity, Instant::now());
                if clicked {
                    send_right_click(state)?;
                }
                state.active_pinch = state
                    .right_pinch
                    .is_active()
                    .then_some(PinchSource::RightClick);
                return Ok(if clicked {
                    PinchAction::RightClick
                } else {
                    PinchAction::None
                });
            }
        }
    }

    let was_scrolling = state.scroll.is_active();
    let vertical_position = two_finger_scroll_position(landmarks, frame_size[1]);
    let update = state.scroll.update(
        vertical_position.is_some(),
        vertical_position,
        state.scroll_sensitivity,
        state.reverse_scroll,
    );
    if update.active {
        state.filter.reset();
        if update.wheel_delta != 0 {
            platform::scroll(update.wheel_delta)?;
        }
        return Ok(PinchAction::Scroll);
    }
    if was_scrolling {
        state.filter.reset();
        return Ok(PinchAction::Cursor);
    }
    Ok(PinchAction::None)
}

impl Drop for CursorController {
    fn drop(&mut self) {
        if Arc::strong_count(&self.state) != 1 {
            return;
        }
        if let Some(state_mutex) = Arc::get_mut(&mut self.state)
            && let Ok(state) = state_mutex.get_mut()
        {
            state.enabled = false;
            let _ = release_left_button(state);
            let _ = release_right_button(state);
        }
    }
}

fn release_left_button(state: &mut CursorState) -> Result<(), String> {
    if !state.left_button_down {
        return Ok(());
    }
    platform::left_button(false)?;
    state.left_button_down = false;
    Ok(())
}

fn release_right_button(state: &mut CursorState) -> Result<(), String> {
    if !state.right_button_down {
        return Ok(());
    }
    platform::right_button(false)?;
    state.right_button_down = false;
    Ok(())
}

fn dispatch_gesture_action(action: PinchAction, state: &mut CursorState) -> Result<(), String> {
    match action {
        PinchAction::None | PinchAction::Cursor | PinchAction::RightClick | PinchAction::Scroll => {
            Ok(())
        }
        PinchAction::Click => send_click(state),
        PinchAction::DragStart => {
            if !state.left_button_down {
                platform::left_button(true)?;
                state.left_button_down = true;
            }
            Ok(())
        }
        PinchAction::DragEnd => release_left_button(state),
    }
}

fn send_right_click(state: &mut CursorState) -> Result<(), String> {
    if state.right_button_down {
        return Err("A mouse button is already held; stop control to release it.".to_owned());
    }
    platform::right_button(true)?;
    state.right_button_down = true;
    platform::right_button(false)?;
    state.right_button_down = false;
    Ok(())
}

fn send_click(state: &mut CursorState) -> Result<(), String> {
    if state.left_button_down {
        return Err("A mouse button is already held; stop control to release it.".to_owned());
    }
    platform::left_button(true)?;
    state.left_button_down = true;
    platform::left_button(false)?;
    state.left_button_down = false;
    Ok(())
}

fn normalized_landmark(point: [f32; 2], frame_size: [u32; 2]) -> Option<[f32; 2]> {
    if frame_size[0] == 0 || frame_size[1] == 0 || !point[0].is_finite() || !point[1].is_finite() {
        return None;
    }
    Some([
        point[0] / frame_size[0] as f32,
        point[1] / frame_size[1] as f32,
    ])
}

fn normalized_pinch_distance(landmarks: &[[f32; 2]; 21], finger_tip: usize) -> Option<f32> {
    let thumb = landmarks[4];
    let fingertip = landmarks[finger_tip];
    let wrist = landmarks[0];
    let middle_mcp = landmarks[9];
    if landmarks
        .iter()
        .flatten()
        .any(|coordinate| !coordinate.is_finite())
    {
        return None;
    }
    let palm_size = distance(wrist, middle_mcp);
    (palm_size > f32::EPSILON).then(|| distance(thumb, fingertip) / palm_size)
}

fn distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
}

fn current_virtual_desktop() -> Result<VirtualDesktop, String> {
    platform::virtual_desktop()
}

#[cfg(windows)]
mod platform {
    use super::VirtualDesktop;
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
        MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    pub fn virtual_desktop() -> Result<VirtualDesktop, String> {
        let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        if width <= 0 || height <= 0 {
            return Err("Windows did not report a usable virtual desktop.".to_owned());
        }
        Ok(VirtualDesktop {
            left,
            top,
            width: width as u32,
            height: height as u32,
        })
    }

    pub fn move_cursor([x, y]: [i32; 2]) -> Result<(), String> {
        send_mouse_input(
            x,
            y,
            0,
            MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
        )
    }

    pub fn left_button(down: bool) -> Result<(), String> {
        let flags = if down {
            MOUSEEVENTF_LEFTDOWN
        } else {
            MOUSEEVENTF_LEFTUP
        };
        send_mouse_input(0, 0, 0, flags)
    }

    pub fn right_button(down: bool) -> Result<(), String> {
        let flags = if down {
            MOUSEEVENTF_RIGHTDOWN
        } else {
            MOUSEEVENTF_RIGHTUP
        };
        send_mouse_input(0, 0, 0, flags)
    }

    pub fn scroll(wheel_delta: i32) -> Result<(), String> {
        send_mouse_input(0, 0, wheel_delta as u32, MOUSEEVENTF_WHEEL)
    }

    fn send_mouse_input(x: i32, y: i32, mouse_data: u32, flags: u32) -> Result<(), String> {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: x,
                    dy: y,
                    mouseData: mouse_data,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let sent = unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) };
        if sent == 1 {
            Ok(())
        } else {
            Err(
                "Windows rejected the input event. Higher-integrity windows may block input."
                    .to_owned(),
            )
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::VirtualDesktop;

    pub fn virtual_desktop() -> Result<VirtualDesktop, String> {
        Err("System cursor control is available only in the Windows desktop app.".to_owned())
    }

    pub fn move_cursor(_position: [i32; 2]) -> Result<(), String> {
        Err("System cursor control is available only in the Windows desktop app.".to_owned())
    }

    pub fn left_button(_down: bool) -> Result<(), String> {
        Err("System cursor control is available only in the Windows desktop app.".to_owned())
    }

    pub fn right_button(_down: bool) -> Result<(), String> {
        Err("System cursor control is available only in the Windows desktop app.".to_owned())
    }

    pub fn scroll(_wheel_delta: i32) -> Result<(), String> {
        Err("System cursor control is available only in the Windows desktop app.".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CursorController, CursorState, normalized_landmark, normalized_pinch_distance,
        release_left_button,
    };

    #[test]
    fn cursor_starts_disabled_and_ignores_tracking_until_explicitly_enabled() {
        let controller = CursorController::default();

        controller
            .update_hand_landmarks(None, [640, 480], 1.0)
            .expect("disabled cursor control must not send system input");
    }

    #[test]
    fn lost_tracking_releases_a_held_mouse_button() {
        let mut state = CursorState {
            enabled: true,
            mirrored: false,
            left_button_down: false,
            ..CursorState::default()
        };

        assert!(release_left_button(&mut state).is_ok());
        assert!(!state.left_button_down);
    }

    #[test]
    fn hand_landmarks_are_normalized_and_invalid_frame_sizes_are_rejected() {
        assert_eq!(
            normalized_landmark([320.0, 120.0], [640, 240]),
            Some([0.5, 0.5])
        );
        assert_eq!(normalized_landmark([0.0, 0.0], [0, 240]), None);
        assert_eq!(normalized_landmark([f32::NAN, 0.0], [640, 240]), None);
    }

    #[test]
    fn pinch_distance_is_scaled_by_the_palm_and_rejects_invalid_landmarks() {
        let mut landmarks = [[0.0, 0.0]; 21];
        landmarks[0] = [0.0, 0.0];
        landmarks[9] = [0.0, 10.0];
        landmarks[4] = [0.0, 4.0];
        landmarks[8] = [0.0, 6.0];
        landmarks[12] = [0.0, 6.0];
        assert_eq!(normalized_pinch_distance(&landmarks, 8), Some(0.2));
        landmarks[12] = [0.0, 8.0];
        assert_eq!(normalized_pinch_distance(&landmarks, 12), Some(0.4));

        landmarks[9] = landmarks[0];
        assert_eq!(normalized_pinch_distance(&landmarks, 8), None);
    }
}
