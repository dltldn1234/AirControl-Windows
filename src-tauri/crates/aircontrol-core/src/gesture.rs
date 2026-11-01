use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinchAction {
    None,
    Cursor,
    Click,
    DragStart,
    DragEnd,
    RightClick,
    Scroll,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinchSource {
    LeftClick,
    RightClick,
}

pub fn choose_pinch_source(
    left_distance: Option<f32>,
    right_distance: Option<f32>,
    sensitivity: f32,
) -> Option<PinchSource> {
    let threshold = 0.28
        * if sensitivity.is_finite() {
            sensitivity.clamp(0.6, 1.4)
        } else {
            1.0
        };
    let left = left_distance.filter(|distance| distance.is_finite() && *distance <= threshold);
    let right = right_distance.filter(|distance| distance.is_finite() && *distance <= threshold);
    match (left, right) {
        (Some(left), Some(right)) if left <= right => Some(PinchSource::LeftClick),
        (Some(_), Some(_)) => Some(PinchSource::RightClick),
        (Some(_), None) => Some(PinchSource::LeftClick),
        (None, Some(_)) => Some(PinchSource::RightClick),
        (None, None) => None,
    }
}

pub fn choose_pinch_source_with_enabled(
    left_distance: Option<f32>,
    right_distance: Option<f32>,
    sensitivity: f32,
    right_click_enabled: bool,
) -> Option<PinchSource> {
    choose_pinch_source(
        left_distance,
        right_click_enabled.then_some(right_distance).flatten(),
        sensitivity,
    )
}

pub fn two_finger_scroll_position(landmarks: &[[f32; 2]; 21], frame_height: u32) -> Option<f32> {
    if frame_height == 0 || landmarks.iter().flatten().any(|value| !value.is_finite()) {
        return None;
    }
    let index_extended = finger_is_extended(landmarks, 5, 6, 8);
    let middle_extended = finger_is_extended(landmarks, 9, 10, 12);
    (index_extended && middle_extended)
        .then(|| (landmarks[8][1] + landmarks[12][1]) / (2.0 * frame_height as f32))
}

fn finger_is_extended(landmarks: &[[f32; 2]; 21], mcp: usize, pip: usize, tip: usize) -> bool {
    let proximal = distance(landmarks[mcp], landmarks[pip]);
    proximal > f32::EPSILON && distance(landmarks[mcp], landmarks[tip]) >= proximal * 1.45
}

fn distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
}

#[derive(Clone, Copy, Debug)]
enum TapState {
    Idle,
    Pinching { started_at: Instant },
}

#[derive(Clone, Copy, Debug)]
pub struct RightClickRecognizer {
    state: TapState,
    last_click_at: Option<Instant>,
    cooldown: Duration,
    maximum_tap_duration: Duration,
}

impl Default for RightClickRecognizer {
    fn default() -> Self {
        Self {
            state: TapState::Idle,
            last_click_at: None,
            cooldown: Duration::from_millis(450),
            maximum_tap_duration: Duration::from_millis(400),
        }
    }
}

impl RightClickRecognizer {
    pub fn is_active(self) -> bool {
        matches!(self.state, TapState::Pinching { .. })
    }

    pub fn update(&mut self, distance: Option<f32>, sensitivity: f32, now: Instant) -> bool {
        let Some(distance) = distance.filter(|distance| distance.is_finite()) else {
            self.state = TapState::Idle;
            self.last_click_at = Some(now);
            return false;
        };
        let sensitivity = if sensitivity.is_finite() {
            sensitivity.clamp(0.6, 1.4)
        } else {
            1.0
        };
        match self.state {
            TapState::Idle => {
                let cooled_down = self
                    .last_click_at
                    .is_none_or(|last| now.saturating_duration_since(last) >= self.cooldown);
                if cooled_down && distance <= 0.28 * sensitivity {
                    self.state = TapState::Pinching { started_at: now };
                }
                false
            }
            TapState::Pinching { started_at } => {
                if distance >= 0.42 * sensitivity {
                    self.state = TapState::Idle;
                    self.last_click_at = Some(now);
                    now.saturating_duration_since(started_at) <= self.maximum_tap_duration
                } else {
                    false
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = TapState::Idle;
    }
}

pub fn next_cursor_gesture(
    current: crate::ActiveGesture,
    action: PinchAction,
) -> crate::ActiveGesture {
    match action {
        PinchAction::Click => crate::ActiveGesture::Click,
        PinchAction::DragStart => crate::ActiveGesture::Drag,
        PinchAction::DragEnd => crate::ActiveGesture::Cursor,
        PinchAction::RightClick => crate::ActiveGesture::RightClick,
        PinchAction::Scroll => crate::ActiveGesture::Scroll,
        PinchAction::Cursor => crate::ActiveGesture::Cursor,
        PinchAction::None
            if matches!(
                current,
                crate::ActiveGesture::Click
                    | crate::ActiveGesture::RightClick
                    | crate::ActiveGesture::Scroll
            ) =>
        {
            crate::ActiveGesture::Cursor
        }
        PinchAction::None => current,
    }
}

#[derive(Clone, Copy, Debug)]
enum PinchState {
    Idle,
    Pinching { started_at: Instant },
    Dragging,
}

/// Separates brief pinches from sustained drags using distance hysteresis and
/// a small cooldown. `None` means tracking was lost and can only end a drag.
#[derive(Clone, Copy, Debug)]
pub struct PinchRecognizer {
    state: PinchState,
    last_action_at: Option<Instant>,
    drag_hold: Duration,
    cooldown: Duration,
}

impl Default for PinchRecognizer {
    fn default() -> Self {
        Self {
            state: PinchState::Idle,
            last_action_at: None,
            drag_hold: Duration::from_millis(450),
            cooldown: Duration::from_millis(350),
        }
    }
}

impl PinchRecognizer {
    pub fn is_active(self) -> bool {
        !matches!(self.state, PinchState::Idle)
    }

    pub fn update(
        &mut self,
        normalized_distance: Option<f32>,
        sensitivity: f32,
        now: Instant,
    ) -> PinchAction {
        let Some(distance) = normalized_distance.filter(|distance| distance.is_finite()) else {
            return self.lose_tracking(now);
        };
        let sensitivity = if sensitivity.is_finite() {
            sensitivity.clamp(0.6, 1.4)
        } else {
            1.0
        };
        let pinch_start = 0.28 * sensitivity;
        let pinch_release = 0.42 * sensitivity;

        match self.state {
            PinchState::Idle => {
                let cooled_down = self
                    .last_action_at
                    .is_none_or(|last| now.saturating_duration_since(last) >= self.cooldown);
                if distance <= pinch_start && cooled_down {
                    self.state = PinchState::Pinching { started_at: now };
                }
                PinchAction::None
            }
            PinchState::Pinching { started_at } => {
                if distance >= pinch_release {
                    self.state = PinchState::Idle;
                    self.last_action_at = Some(now);
                    PinchAction::Click
                } else if now.saturating_duration_since(started_at) >= self.drag_hold {
                    self.state = PinchState::Dragging;
                    self.last_action_at = Some(now);
                    PinchAction::DragStart
                } else {
                    PinchAction::None
                }
            }
            PinchState::Dragging => {
                if distance >= pinch_release {
                    self.state = PinchState::Idle;
                    self.last_action_at = Some(now);
                    PinchAction::DragEnd
                } else {
                    PinchAction::None
                }
            }
        }
    }

    pub fn reset(&mut self) -> PinchAction {
        self.lose_tracking(Instant::now())
    }

    fn lose_tracking(&mut self, now: Instant) -> PinchAction {
        let was_dragging = matches!(self.state, PinchState::Dragging);
        self.state = PinchState::Idle;
        if was_dragging {
            self.last_action_at = Some(now);
            PinchAction::DragEnd
        } else {
            PinchAction::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PinchAction, PinchRecognizer, PinchSource, RightClickRecognizer, choose_pinch_source,
        choose_pinch_source_with_enabled, next_cursor_gesture, two_finger_scroll_position,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn short_pinch_emits_one_click_after_release_with_hysteresis() {
        let start = Instant::now();
        let mut recognizer = PinchRecognizer::default();
        assert_eq!(recognizer.update(Some(0.2), 1.0, start), PinchAction::None);
        assert_eq!(
            recognizer.update(Some(0.34), 1.0, start + Duration::from_millis(80)),
            PinchAction::None
        );
        assert_eq!(
            recognizer.update(Some(0.5), 1.0, start + Duration::from_millis(110)),
            PinchAction::Click
        );
        assert_eq!(
            recognizer.update(Some(0.1), 1.0, start + Duration::from_millis(120)),
            PinchAction::None
        );
    }

    #[test]
    fn sustained_pinch_starts_a_drag_and_release_or_tracking_loss_ends_it() {
        let start = Instant::now();
        let mut recognizer = PinchRecognizer::default();
        assert_eq!(recognizer.update(Some(0.1), 1.0, start), PinchAction::None);
        assert_eq!(
            recognizer.update(Some(0.2), 1.0, start + Duration::from_millis(460)),
            PinchAction::DragStart
        );
        assert_eq!(
            recognizer.update(None, 1.0, start + Duration::from_millis(470)),
            PinchAction::DragEnd
        );
    }

    #[test]
    fn non_finite_distances_are_treated_as_tracking_loss() {
        let mut recognizer = PinchRecognizer::default();
        assert_eq!(
            recognizer.update(Some(f32::NAN), 1.0, Instant::now()),
            PinchAction::None
        );
    }

    #[test]
    fn lost_drag_releases_and_applies_cooldown_before_rearming() {
        let start = Instant::now();
        let mut recognizer = PinchRecognizer::default();
        assert_eq!(recognizer.update(Some(0.1), 1.0, start), PinchAction::None);
        assert_eq!(
            recognizer.update(Some(0.1), 1.0, start + Duration::from_millis(460)),
            PinchAction::DragStart
        );
        assert_eq!(
            recognizer.update(None, 1.0, start + Duration::from_millis(470)),
            PinchAction::DragEnd
        );
        assert_eq!(
            recognizer.update(Some(0.1), 1.0, start + Duration::from_millis(500)),
            PinchAction::None
        );
    }

    #[test]
    fn cursor_status_tracks_click_and_drag_transitions() {
        use crate::ActiveGesture;

        assert_eq!(
            next_cursor_gesture(ActiveGesture::Cursor, PinchAction::Click),
            ActiveGesture::Click
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::Click, PinchAction::None),
            ActiveGesture::Cursor
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::Drag, PinchAction::None),
            ActiveGesture::Drag
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::Drag, PinchAction::DragEnd),
            ActiveGesture::Cursor
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::Cursor, PinchAction::RightClick),
            ActiveGesture::RightClick
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::RightClick, PinchAction::None),
            ActiveGesture::Cursor
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::Cursor, PinchAction::Scroll),
            ActiveGesture::Scroll
        );
        assert_eq!(
            next_cursor_gesture(ActiveGesture::Scroll, PinchAction::Cursor),
            ActiveGesture::Cursor
        );
    }

    #[test]
    fn conflicting_pinches_choose_the_closer_pair_and_prefer_left_on_a_tie() {
        assert_eq!(
            choose_pinch_source(Some(0.12), Some(0.2), 1.0),
            Some(PinchSource::LeftClick)
        );
        assert_eq!(
            choose_pinch_source(Some(0.2), Some(0.12), 1.0),
            Some(PinchSource::RightClick)
        );
        assert_eq!(
            choose_pinch_source(Some(0.15), Some(0.15), 1.0),
            Some(PinchSource::LeftClick)
        );
    }

    #[test]
    fn disabled_right_click_does_not_select_the_middle_finger_pinch() {
        assert_eq!(
            choose_pinch_source_with_enabled(Some(0.5), Some(0.12), 1.0, false),
            None,
            "a middle-finger pinch is ignored when right click is disabled"
        );
        assert_eq!(
            choose_pinch_source_with_enabled(Some(0.12), Some(0.12), 1.0, true),
            Some(PinchSource::LeftClick)
        );
    }

    #[test]
    fn short_thumb_middle_tap_emits_once_after_release() {
        let start = Instant::now();
        let mut recognizer = RightClickRecognizer::default();
        assert!(!recognizer.update(Some(0.2), 1.0, start));
        assert!(!recognizer.update(Some(0.34), 1.0, start + Duration::from_millis(90)));
        assert!(recognizer.update(Some(0.5), 1.0, start + Duration::from_millis(140)));
        assert!(!recognizer.update(Some(0.1), 1.0, start + Duration::from_millis(150)));
    }

    #[test]
    fn long_right_pinch_and_tracking_loss_do_not_click() {
        let start = Instant::now();
        let mut held = RightClickRecognizer::default();
        assert!(!held.update(Some(0.1), 1.0, start));
        assert!(!held.update(Some(0.1), 1.0, start + Duration::from_millis(500)));
        assert!(!held.update(Some(0.5), 1.0, start + Duration::from_millis(510)));

        let mut lost = RightClickRecognizer::default();
        assert!(!lost.update(Some(0.1), 1.0, start));
        assert!(!lost.update(None, 1.0, start + Duration::from_millis(120)));
    }

    #[test]
    fn scrolling_requires_both_fingers_extended_and_returns_normalized_vertical_position() {
        let mut landmarks = [[0.0, 0.0]; 21];
        landmarks[5] = [0.0, 0.0];
        landmarks[6] = [0.0, 4.0];
        landmarks[8] = [0.0, 10.0];
        landmarks[9] = [5.0, 0.0];
        landmarks[10] = [5.0, 4.0];
        landmarks[12] = [5.0, 12.0];
        assert_eq!(
            two_finger_scroll_position(&landmarks, 24),
            Some(11.0 / 24.0)
        );

        landmarks[12] = [7.0, 3.0];
        assert_eq!(two_finger_scroll_position(&landmarks, 24), None);
        assert_eq!(two_finger_scroll_position(&landmarks, 0), None);
    }
}
