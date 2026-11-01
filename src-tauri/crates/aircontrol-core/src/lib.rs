use crate::cursor::CursorCalibration;
use serde::{Deserialize, Serialize};

pub mod app_profile;
pub mod app_switch;
pub mod command_palette;
pub mod cursor;
pub mod filter;
pub mod gesture;
pub mod hand_tracking;
pub mod keyboard;
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
    CommandPalette,
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
    WindowResize,
    AppSwitch,
    CommandPalette,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub camera: CameraState,
    pub hand_tracking: HandTrackingState,
    pub control_mode: ControlMode,
    pub active_gesture: ActiveGesture,
    pub frames_per_second: Option<f32>,
    pub active_profile_id: Option<String>,
    pub palette_hovered_index: Option<usize>,
    pub palette_message: Option<String>,
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
    pub app_switch_enabled: bool,
    pub cursor_calibration: CursorCalibration,
    pub active_profile_id: String,
    pub profiles: Vec<ControlProfile>,
    pub auto_profile_enabled: bool,
    pub app_profile_rules: Vec<app_profile::AppProfileRule>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlProfile {
    pub id: String,
    pub name: String,
    pub built_in: bool,
    pub settings: ProfileSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettings {
    pub cursor_sensitivity: f32,
    pub cursor_smoothing: f32,
    pub pinch_sensitivity: f32,
    pub window_sensitivity: f32,
    pub scroll_sensitivity: f32,
    pub reverse_scroll_direction: bool,
    pub right_click_enabled: bool,
    pub camera_mirrored: bool,
    pub app_switch_enabled: bool,
}

impl ProfileSettings {
    fn normalized(mut self) -> Self {
        self.cursor_sensitivity = finite_clamp(self.cursor_sensitivity, 0.5, 2.0, 1.0);
        self.cursor_smoothing = finite_clamp(self.cursor_smoothing, 0.0, 0.9, 0.4);
        self.pinch_sensitivity = finite_clamp(self.pinch_sensitivity, 0.6, 1.4, 1.0);
        self.window_sensitivity = finite_clamp(self.window_sensitivity, 0.4, 2.0, 1.0);
        self.scroll_sensitivity = finite_clamp(self.scroll_sensitivity, 0.4, 3.0, 1.0);
        self
    }
}

impl ControlProfile {
    pub fn built_ins() -> Vec<Self> {
        let base = ProfileSettings::default();
        let mut presentation = base.clone();
        presentation.cursor_smoothing = 0.55;
        presentation.app_switch_enabled = true;
        let mut custom = base.clone();
        custom.cursor_sensitivity = 1.2;
        vec![
            Self {
                id: "productivity".into(),
                name: "Productivity".into(),
                built_in: true,
                settings: base,
            },
            Self {
                id: "presentation".into(),
                name: "Presentation".into(),
                built_in: true,
                settings: presentation,
            },
            Self {
                id: "custom".into(),
                name: "Custom".into(),
                built_in: true,
                settings: custom,
            },
        ]
    }
}

impl Default for ProfileSettings {
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
            app_switch_enabled: false,
        }
    }
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
            app_switch_enabled: false,
            cursor_calibration: CursorCalibration::default(),
            active_profile_id: "productivity".into(),
            profiles: ControlProfile::built_ins(),
            auto_profile_enabled: false,
            app_profile_rules: Vec::new(),
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
        self.cursor_calibration = self.cursor_calibration.normalized();
        if self.profiles.is_empty() {
            self.profiles = ControlProfile::built_ins();
        }
        for profile in &mut self.profiles {
            profile.settings = profile.settings.clone().normalized();
        }
        if !self
            .profiles
            .iter()
            .any(|profile| profile.id == self.active_profile_id)
        {
            self.active_profile_id = "productivity".into();
        }
        let active_settings = ProfileSettings {
            cursor_sensitivity: self.cursor_sensitivity,
            cursor_smoothing: self.cursor_smoothing,
            pinch_sensitivity: self.pinch_sensitivity,
            window_sensitivity: self.window_sensitivity,
            scroll_sensitivity: self.scroll_sensitivity,
            reverse_scroll_direction: self.reverse_scroll_direction,
            right_click_enabled: self.right_click_enabled,
            camera_mirrored: self.camera_mirrored,
            app_switch_enabled: self.app_switch_enabled,
        };
        if let Some(active) = self
            .profiles
            .iter_mut()
            .find(|profile| profile.id == self.active_profile_id)
        {
            active.settings = active_settings;
        }
        self
    }

    pub fn select_profile(mut self, profile_id: &str) -> Result<Self, String> {
        let profile = self
            .profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .ok_or_else(|| "The selected control profile does not exist.".to_owned())?;
        let settings = profile.settings.clone();
        self.active_profile_id = profile.id.clone();
        self.cursor_sensitivity = settings.cursor_sensitivity;
        self.cursor_smoothing = settings.cursor_smoothing;
        self.pinch_sensitivity = settings.pinch_sensitivity;
        self.window_sensitivity = settings.window_sensitivity;
        self.scroll_sensitivity = settings.scroll_sensitivity;
        self.reverse_scroll_direction = settings.reverse_scroll_direction;
        self.right_click_enabled = settings.right_click_enabled;
        self.camera_mirrored = settings.camera_mirrored;
        self.app_switch_enabled = settings.app_switch_enabled;
        Ok(self)
    }

    pub fn validate_profiles(&self) -> Result<(), String> {
        let mut ids = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for profile in &self.profiles {
            if profile.id.trim().is_empty()
                || profile.name.trim().is_empty()
                || profile.name.chars().count() > 32
            {
                return Err("Profile names must contain 1 to 32 characters.".to_owned());
            }
            if !ids.insert(profile.id.trim().to_ascii_lowercase()) {
                return Err("Profile identifiers must be unique.".to_owned());
            }
            if !names.insert(profile.name.trim().to_lowercase()) {
                return Err("Profile names must be unique.".to_owned());
            }
            if profile.built_in
                && !ControlProfile::built_ins()
                    .iter()
                    .any(|builtin| builtin.id == profile.id)
            {
                return Err("Unknown profiles cannot be marked as built-in.".to_owned());
            }
        }
        for builtin in ControlProfile::built_ins() {
            if !self.profiles.iter().any(|profile| {
                profile.id == builtin.id && profile.built_in && profile.name == builtin.name
            }) {
                return Err("Built-in profiles must remain available.".to_owned());
            }
        }
        if !self
            .profiles
            .iter()
            .any(|profile| profile.id == self.active_profile_id)
        {
            return Err("The active profile must exist.".to_owned());
        }
        app_profile::validate_app_profile_rules(self)
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

    #[test]
    fn built_in_profiles_are_available_without_enabling_system_control() {
        let preferences = UserPreferences::default();

        assert_eq!(
            preferences
                .profiles
                .iter()
                .filter(|profile| profile.built_in)
                .count(),
            3
        );
        assert_eq!(preferences.active_profile_id, "productivity");
        assert_eq!(ControlMode::default(), ControlMode::Disabled);
    }

    #[test]
    fn selecting_profile_applies_its_settings_and_keeps_selection_persistent() {
        let preferences = UserPreferences::default()
            .select_profile("presentation")
            .expect("presentation preset");

        assert_eq!(preferences.active_profile_id, "presentation");
        assert!(preferences.app_switch_enabled);
        assert_eq!(preferences.cursor_smoothing, 0.55);
        assert!(preferences.validate_profiles().is_ok());
    }

    #[test]
    fn selecting_unknown_profile_returns_an_error_without_changing_preferences() {
        let preferences = UserPreferences::default();

        assert!(preferences.clone().select_profile("missing").is_err());
        assert_eq!(preferences.active_profile_id, "productivity");
    }

    #[test]
    fn duplicate_or_missing_builtin_profiles_are_rejected() {
        let mut duplicate = UserPreferences::default();
        duplicate.profiles[1].name = duplicate.profiles[0].name.clone();
        assert!(duplicate.validate_profiles().is_err());

        let mut missing_builtin = UserPreferences::default();
        missing_builtin
            .profiles
            .retain(|profile| profile.id != "presentation");
        assert!(missing_builtin.validate_profiles().is_err());

        let mut renamed_builtin = UserPreferences::default();
        renamed_builtin.profiles[0].name = "My productivity".into();
        assert!(renamed_builtin.validate_profiles().is_err());
    }
}
