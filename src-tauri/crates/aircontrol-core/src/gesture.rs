use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinchAction {
    None,
    Click,
    DragStart,
    DragEnd,
}

pub fn next_cursor_gesture(
    current: crate::ActiveGesture,
    action: PinchAction,
) -> crate::ActiveGesture {
    match action {
        PinchAction::Click => crate::ActiveGesture::Click,
        PinchAction::DragStart => crate::ActiveGesture::Drag,
        PinchAction::DragEnd => crate::ActiveGesture::Cursor,
        PinchAction::None if current == crate::ActiveGesture::Click => crate::ActiveGesture::Cursor,
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
    use super::{PinchAction, PinchRecognizer, next_cursor_gesture};
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
    }
}
