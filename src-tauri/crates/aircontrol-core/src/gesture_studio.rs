use crate::app_switch::{AppSwitchSwipeRecognizer, three_finger_swipe_position};
use crate::gesture::{
    PinchAction, PinchRecognizer, PinchSource, RightClickRecognizer,
    choose_pinch_source_with_sensitivities, two_finger_scroll_position,
};
use crate::scroll::ScrollRecognizer;
use crate::{ActiveGesture, ControlMode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GestureId {
    IndexPinch,
    MiddlePinch,
    TwoFingerScroll,
    ThreeFingerSwipe,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GestureAction {
    ClickAndDrag,
    Click,
    Drag,
    RightClick,
    Scroll,
    SwitchApplication,
    Disabled,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GestureBinding {
    pub gesture: GestureId,
    pub action: GestureAction,
    pub sensitivity: f32,
    pub cooldown_ms: u64,
}

impl GestureBinding {
    pub fn enabled(self) -> bool {
        self.action != GestureAction::Disabled
    }

    fn normalized(mut self) -> Self {
        self.sensitivity = if self.sensitivity.is_finite() {
            self.sensitivity.clamp(0.6, 1.4)
        } else {
            1.0
        };
        self.cooldown_ms = self.cooldown_ms.clamp(100, 2_000);
        self
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct GestureStudioSettings {
    pub bindings: Vec<GestureBinding>,
}

impl Default for GestureStudioSettings {
    fn default() -> Self {
        Self {
            bindings: vec![
                GestureBinding {
                    gesture: GestureId::IndexPinch,
                    action: GestureAction::ClickAndDrag,
                    sensitivity: 1.0,
                    cooldown_ms: 350,
                },
                GestureBinding {
                    gesture: GestureId::MiddlePinch,
                    action: GestureAction::RightClick,
                    sensitivity: 1.0,
                    cooldown_ms: 450,
                },
                GestureBinding {
                    gesture: GestureId::TwoFingerScroll,
                    action: GestureAction::Scroll,
                    sensitivity: 1.0,
                    cooldown_ms: 250,
                },
                GestureBinding {
                    gesture: GestureId::ThreeFingerSwipe,
                    action: GestureAction::SwitchApplication,
                    sensitivity: 1.0,
                    cooldown_ms: 900,
                },
            ],
        }
    }
}

impl GestureStudioSettings {
    pub fn binding(&self, gesture: GestureId) -> GestureBinding {
        self.bindings
            .iter()
            .find(|binding| binding.gesture == gesture)
            .copied()
            .unwrap_or_else(|| {
                Self::default()
                    .bindings
                    .into_iter()
                    .find(|binding| binding.gesture == gesture)
                    .expect("all gesture IDs have a default binding")
            })
    }

    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();
        let mut seen = HashSet::new();
        self.bindings.retain(|binding| seen.insert(binding.gesture));
        for default in defaults.bindings {
            if let Some(binding) = self
                .bindings
                .iter_mut()
                .find(|binding| binding.gesture == default.gesture)
            {
                binding.sensitivity = binding.normalized().sensitivity;
                binding.cooldown_ms = binding.normalized().cooldown_ms;
                if !action_is_valid_for(binding.gesture, binding.action) {
                    binding.action = default.action;
                }
            } else {
                self.bindings.push(default);
            }
        }
        self.bindings.sort_by_key(|binding| binding.gesture as u8);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.bindings.len() != 4 {
            return Err("Each gesture must have exactly one action binding.".to_owned());
        }
        let mut gestures = HashSet::new();
        let mut actions = HashSet::new();
        for binding in &self.bindings {
            if !gestures.insert(binding.gesture) {
                return Err("A gesture cannot be assigned more than once.".to_owned());
            }
            if !action_is_valid_for(binding.gesture, binding.action) {
                return Err("The selected action is not supported by this gesture.".to_owned());
            }
            if !binding.sensitivity.is_finite() || !(0.6..=1.4).contains(&binding.sensitivity) {
                return Err("Gesture sensitivity must be between 0.6 and 1.4.".to_owned());
            }
            if !(100..=2_000).contains(&binding.cooldown_ms) {
                return Err("Gesture cooldown must be between 100 and 2000 ms.".to_owned());
            }
            if binding.enabled() && !actions.insert(binding.action) {
                return Err("Two gestures cannot be assigned to the same action.".to_owned());
            }
        }
        Ok(())
    }

    pub fn has_conflicts(&self) -> bool {
        self.validate().is_err()
    }
}

fn action_is_valid_for(gesture: GestureId, action: GestureAction) -> bool {
    match gesture {
        GestureId::IndexPinch => matches!(
            action,
            GestureAction::ClickAndDrag
                | GestureAction::Click
                | GestureAction::Drag
                | GestureAction::Disabled
        ),
        GestureId::MiddlePinch => {
            matches!(action, GestureAction::RightClick | GestureAction::Disabled)
        }
        GestureId::TwoFingerScroll => {
            matches!(action, GestureAction::Scroll | GestureAction::Disabled)
        }
        GestureId::ThreeFingerSwipe => {
            matches!(
                action,
                GestureAction::SwitchApplication | GestureAction::Disabled
            )
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GestureTestUpdate {
    pub active_gesture: ActiveGesture,
    pub expected_action: Option<GestureAction>,
    pub tracking_lost: bool,
    pub confidence: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct GestureTestSession {
    active: bool,
    active_pinch: Option<PinchSource>,
    pinch: PinchRecognizer,
    right_pinch: RightClickRecognizer,
    scroll: ScrollRecognizer,
    app_switch: AppSwitchSwipeRecognizer,
}

impl GestureTestSession {
    pub fn start(&mut self) {
        self.reset();
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.reset();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn update(
        &mut self,
        landmarks: Option<[[f32; 2]; 21]>,
        frame_size: [u32; 2],
        mirrored: bool,
        confidence: Option<f32>,
        settings: &GestureStudioSettings,
        now: Instant,
    ) -> GestureTestUpdate {
        if !self.active {
            return GestureTestUpdate::default();
        }
        let Some(landmarks) = landmarks.filter(|points| {
            frame_size[0] > 0
                && frame_size[1] > 0
                && points
                    .iter()
                    .flatten()
                    .all(|coordinate| coordinate.is_finite())
        }) else {
            self.reset_recognizers();
            return GestureTestUpdate {
                tracking_lost: true,
                ..GestureTestUpdate::default()
            };
        };

        let index = settings.binding(GestureId::IndexPinch);
        let middle = settings.binding(GestureId::MiddlePinch);
        let (index_distance, middle_distance) = normalized_pinch_distances(&landmarks);
        let source = self.active_pinch.or_else(|| {
            choose_pinch_source_with_sensitivities(
                index.enabled().then_some(index_distance).flatten(),
                index.sensitivity,
                middle.enabled().then_some(middle_distance).flatten(),
                middle.sensitivity,
            )
        });
        match source {
            Some(PinchSource::LeftClick) => {
                self.active_pinch = Some(PinchSource::LeftClick);
                self.right_pinch.reset();
                self.scroll.reset();
                self.app_switch.reset();
                let action = self.pinch.update_with_timing(
                    index_distance,
                    index.sensitivity,
                    Duration::from_millis(450),
                    Duration::from_millis(index.cooldown_ms),
                    now,
                );
                let mapped = map_index_action(index.action, action);
                let dragging = self.pinch.is_dragging();
                let expected_action = mapped.or_else(|| {
                    (self.pinch.is_active() && !dragging && index.action != GestureAction::Drag)
                        .then_some(GestureAction::Click)
                });
                if !self.pinch.is_active() {
                    self.active_pinch = None;
                }
                return GestureTestUpdate {
                    active_gesture: if dragging || mapped == Some(GestureAction::Drag) {
                        ActiveGesture::Drag
                    } else if expected_action == Some(GestureAction::Click) {
                        ActiveGesture::Click
                    } else {
                        ActiveGesture::None
                    },
                    expected_action,
                    tracking_lost: false,
                    confidence: valid_confidence(confidence),
                };
            }
            Some(PinchSource::RightClick) => {
                self.active_pinch = Some(PinchSource::RightClick);
                let _ = self.pinch.reset();
                self.scroll.reset();
                self.app_switch.reset();
                let clicked = self.right_pinch.update_with_cooldown(
                    middle_distance,
                    middle.sensitivity,
                    Duration::from_millis(middle.cooldown_ms),
                    now,
                );
                if !self.right_pinch.is_active() {
                    self.active_pinch = None;
                }
                return GestureTestUpdate {
                    active_gesture: if clicked || self.right_pinch.is_active() {
                        ActiveGesture::RightClick
                    } else {
                        ActiveGesture::None
                    },
                    expected_action: clicked.then_some(GestureAction::RightClick),
                    tracking_lost: false,
                    confidence: valid_confidence(confidence),
                };
            }
            None => {}
        }

        self.pinch.reset();
        self.right_pinch.reset();
        self.active_pinch = None;
        let scroll_binding = settings.binding(GestureId::TwoFingerScroll);
        if scroll_binding.enabled() {
            let position = two_finger_scroll_position(&landmarks, frame_size[1]);
            let result = self.scroll.update(
                position.is_some(),
                position,
                scroll_binding.sensitivity,
                false,
            );
            if result.active {
                self.app_switch.reset();
                return GestureTestUpdate {
                    active_gesture: ActiveGesture::Scroll,
                    expected_action: (result.wheel_delta != 0).then_some(GestureAction::Scroll),
                    tracking_lost: false,
                    confidence: valid_confidence(confidence),
                };
            }
        } else {
            self.scroll.reset();
        }

        let swipe = settings.binding(GestureId::ThreeFingerSwipe);
        if swipe.enabled() {
            let position = three_finger_swipe_position(&landmarks, frame_size, mirrored);
            let direction = self.app_switch.update_with_settings(
                position,
                now,
                swipe.sensitivity,
                Duration::from_millis(swipe.cooldown_ms),
            );
            if direction.is_some() {
                return GestureTestUpdate {
                    active_gesture: ActiveGesture::AppSwitch,
                    expected_action: Some(GestureAction::SwitchApplication),
                    tracking_lost: false,
                    confidence: valid_confidence(confidence),
                };
            }
            if position.is_some() {
                return GestureTestUpdate {
                    active_gesture: ActiveGesture::AppSwitch,
                    expected_action: None,
                    tracking_lost: false,
                    confidence: valid_confidence(confidence),
                };
            }
        } else {
            self.app_switch.reset();
        }

        GestureTestUpdate {
            confidence: valid_confidence(confidence),
            ..GestureTestUpdate::default()
        }
    }

    fn reset(&mut self) {
        self.active = false;
        self.reset_recognizers();
    }

    fn reset_recognizers(&mut self) {
        self.pinch.reset();
        self.right_pinch.reset();
        self.active_pinch = None;
        self.scroll.reset();
        self.app_switch.reset();
    }
}

pub const fn system_input_allowed(mode: ControlMode) -> bool {
    !matches!(mode, ControlMode::Disabled | ControlMode::GestureTest)
}

fn map_index_action(action: GestureAction, pinch: PinchAction) -> Option<GestureAction> {
    match (action, pinch) {
        (GestureAction::ClickAndDrag, PinchAction::Click) => Some(GestureAction::Click),
        (GestureAction::ClickAndDrag, PinchAction::DragStart) => Some(GestureAction::Drag),
        (GestureAction::Click, PinchAction::Click) => Some(GestureAction::Click),
        (GestureAction::Drag, PinchAction::DragStart) => Some(GestureAction::Drag),
        _ => None,
    }
}

fn normalized_pinch_distances(landmarks: &[[f32; 2]; 21]) -> (Option<f32>, Option<f32>) {
    let scale = point_distance(landmarks[0], landmarks[9]);
    if !scale.is_finite() || scale <= f32::EPSILON {
        return (None, None);
    }
    (
        Some(point_distance(landmarks[4], landmarks[8]) / scale),
        Some(point_distance(landmarks[4], landmarks[12]) / scale),
    )
}

fn point_distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
}

fn valid_confidence(confidence: Option<f32>) -> Option<f32> {
    confidence
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveGesture, GestureAction, GestureBinding, GestureId, GestureStudioSettings,
        GestureTestSession, system_input_allowed,
    };
    use crate::ControlMode;
    use std::time::{Duration, Instant};

    #[test]
    fn default_bindings_are_unique_and_validation_rejects_collisions() {
        let mut settings = GestureStudioSettings::default();
        assert!(settings.validate().is_ok());
        settings.bindings[1].action = settings.bindings[0].action;
        assert!(settings.has_conflicts());
        assert!(settings.validate().is_err());
    }

    #[test]
    fn legacy_settings_normalize_missing_duplicate_and_out_of_range_values() {
        let mut settings = GestureStudioSettings::default();
        settings.bindings.pop();
        settings.bindings[0].sensitivity = f32::NAN;
        settings.bindings[0].cooldown_ms = 9_000;
        settings.bindings.push(settings.bindings[0]);
        let normalized = settings.normalized();
        assert_eq!(normalized.bindings.len(), 4);
        assert_eq!(normalized.bindings[0].sensitivity, 1.0);
        assert_eq!(normalized.bindings[0].cooldown_ms, 2_000);
    }

    #[test]
    fn safe_testing_mode_never_allows_system_input_dispatch() {
        assert!(!system_input_allowed(ControlMode::GestureTest));
        assert!(!system_input_allowed(ControlMode::Disabled));
        assert!(system_input_allowed(ControlMode::Cursor));
    }

    #[test]
    fn stop_and_tracking_loss_reset_all_test_gesture_state() {
        let mut session = GestureTestSession::default();
        session.start();
        let lost = session.update(
            None,
            [640, 480],
            false,
            None,
            &GestureStudioSettings::default(),
            Instant::now(),
        );
        assert!(lost.tracking_lost);
        assert!(session.is_active());
        session.stop();
        assert!(!session.is_active());
        assert!(
            !session
                .update(
                    None,
                    [640, 480],
                    false,
                    None,
                    &GestureStudioSettings::default(),
                    Instant::now()
                )
                .tracking_lost
        );
    }

    #[test]
    fn tracking_loss_releases_the_active_pinch_source_before_reacquisition() {
        let mut session = GestureTestSession::default();
        session.start();
        let now = Instant::now();
        let mut landmarks = [[0.0; 2]; 21];
        landmarks[9] = [0.0, 100.0];
        landmarks[4] = [10.0, 0.0];
        landmarks[8] = [12.0, 0.0];
        landmarks[12] = [90.0, 0.0];
        let settings = GestureStudioSettings::default();
        let index_pinch = session.update(
            Some(landmarks),
            [640, 480],
            false,
            Some(0.9),
            &settings,
            now,
        );
        assert_eq!(index_pinch.active_gesture, ActiveGesture::Click);
        let lost = session.update(
            None,
            [640, 480],
            false,
            None,
            &settings,
            now + Duration::from_millis(20),
        );
        assert!(lost.tracking_lost);

        landmarks[8] = [80.0, 0.0];
        landmarks[12] = [12.0, 0.0];
        let middle_pinch = session.update(
            Some(landmarks),
            [640, 480],
            false,
            Some(0.9),
            &settings,
            now + Duration::from_millis(40),
        );
        assert_eq!(middle_pinch.active_gesture, ActiveGesture::RightClick);
    }

    #[test]
    fn short_index_pinch_reports_a_predicted_click_without_native_input() {
        let mut session = GestureTestSession::default();
        session.start();
        let now = Instant::now();
        let mut landmarks = [[0.0; 2]; 21];
        landmarks[0] = [0.0, 0.0];
        landmarks[9] = [0.0, 100.0];
        landmarks[4] = [10.0, 0.0];
        landmarks[8] = [8.0, 0.0];
        let settings = GestureStudioSettings::default();
        let first = session.update(
            Some(landmarks),
            [640, 480],
            false,
            Some(0.92),
            &settings,
            now,
        );
        assert_eq!(first.expected_action, Some(GestureAction::Click));
        landmarks[8] = [60.0, 0.0];
        let release = session.update(
            Some(landmarks),
            [640, 480],
            false,
            Some(0.92),
            &settings,
            now + Duration::from_millis(80),
        );
        assert_eq!(release.expected_action, Some(GestureAction::Click));
    }

    #[test]
    fn unsupported_action_assignments_are_rejected() {
        let mut settings = GestureStudioSettings::default();
        settings.bindings[2] = GestureBinding {
            gesture: GestureId::TwoFingerScroll,
            action: GestureAction::Click,
            sensitivity: 1.0,
            cooldown_ms: 250,
        };
        assert!(settings.validate().is_err());
    }
}
