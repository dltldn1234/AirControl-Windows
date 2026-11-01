use serde::{Deserialize, Serialize};

pub mod cursor;
pub mod filter;
pub mod gesture;
pub mod scroll;
pub mod window;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CameraState {
    #[default]
    Stopped,
    Starting,
    Running,
    RequestingPermission,
    Enumerating,
    PermissionRequired,
    Unavailable,
    Error,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HandTrackingState {
    #[default]
    Waiting,
    ModelLoading,
    ModelReady,
    PalmDetected,
    Detected,
    Lost,
    Error,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlMode {
    #[default]
    Disabled,
    Cursor,
    Window,
    AirKeyboard,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ActiveGesture {
    #[default]
    None,
    Cursor,
    Click,
    Drag,
    Scroll,
    RightClick,
    WindowMove,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub camera: CameraState,
    pub hand_tracking: HandTrackingState,
    pub control_mode: ControlMode,
    pub active_gesture: ActiveGesture,
    pub frames_per_second: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ControlReadiness {
    pub camera_running: bool,
    pub hand_pose_detected: bool,
}

impl ControlReadiness {
    pub const fn can_enable(self) -> bool {
        self.camera_running && self.hand_pose_detected
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct UserPreferences {
    pub cursor_sensitivity: f32,
    pub cursor_smoothing: f32,
    pub pinch_sensitivity: f32,
    pub window_sensitivity: f32,
    pub scroll_sensitivity: f32,
    pub reverse_scroll_direction: bool,
    pub right_click_enabled: bool,
    pub camera_mirrored: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            cursor_sensitivity: 1.0,
            cursor_smoothing: 0.4,
            pinch_sensitivity: 1.0,
            window_sensitivity: 1.0,
            scroll_sensitivity: 1.0,
            reverse_scroll_direction: false,
            right_click_enabled: true,
            camera_mirrored: true,
        }
    }
}

impl UserPreferences {
    pub fn normalized(mut self) -> Self {
        self.cursor_sensitivity = finite_clamp(self.cursor_sensitivity, 0.5, 2.0, 1.0);
        self.cursor_smoothing = finite_clamp(self.cursor_smoothing, 0.0, 0.9, 0.4);
        self.pinch_sensitivity = finite_clamp(self.pinch_sensitivity, 0.6, 1.4, 1.0);
        self.window_sensitivity = finite_clamp(self.window_sensitivity, 0.4, 2.0, 1.0);
        self.scroll_sensitivity = finite_clamp(self.scroll_sensitivity, 0.4, 3.0, 1.0);
        self
    }
}

fn finite_clamp(value: f32, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(minimum, maximum)
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlMode, ControlReadiness, RuntimeSnapshot, UserPreferences};

    #[test]
    fn runtime_starts_with_all_system_control_disabled() {
        let status = RuntimeSnapshot::default();

        assert_eq!(status.control_mode, ControlMode::Disabled);
        assert_eq!(status.frames_per_second, None);
    }

    #[test]
    fn cursor_control_requires_camera_and_a_detected_hand_pose() {
        assert!(!ControlReadiness::default().can_enable());
        assert!(
            !ControlReadiness {
                camera_running: true,
                hand_pose_detected: false,
            }
            .can_enable()
        );
        assert!(
            !ControlReadiness {
                camera_running: false,
                hand_pose_detected: true,
            }
            .can_enable()
        );
        assert!(
            ControlReadiness {
                camera_running: true,
                hand_pose_detected: true,
            }
            .can_enable()
        );
    }

    #[test]
    fn preference_sensitivities_are_sanitized_to_supported_ranges() {
        let normalized = UserPreferences {
            cursor_sensitivity: f32::INFINITY,
            cursor_smoothing: -4.0,
            pinch_sensitivity: 8.0,
            window_sensitivity: 0.0,
            scroll_sensitivity: f32::NAN,
            ..UserPreferences::default()
        }
        .normalized();

        assert_eq!(normalized.cursor_sensitivity, 1.0);
        assert_eq!(normalized.cursor_smoothing, 0.0);
        assert_eq!(normalized.pinch_sensitivity, 1.4);
        assert_eq!(normalized.window_sensitivity, 0.4);
        assert_eq!(normalized.scroll_sensitivity, 1.0);
    }
}
