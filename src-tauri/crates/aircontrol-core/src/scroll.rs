const SCROLL_UNITS_PER_NORMALIZED_POINT: f32 = 1200.0;
const MINIMUM_SCROLL_MOTION: f32 = 0.002;
const MAX_WHEEL_DELTA_PER_SAMPLE: i32 = 480;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScrollUpdate {
    pub active: bool,
    pub wheel_delta: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollRecognizer {
    previous_y: Option<f32>,
    pending_motion: f32,
}

impl ScrollRecognizer {
    pub fn is_active(self) -> bool {
        self.previous_y.is_some()
    }

    pub fn update(
        &mut self,
        two_fingers_extended: bool,
        normalized_y: Option<f32>,
        sensitivity: f32,
        reversed: bool,
    ) -> ScrollUpdate {
        let Some(y) = normalized_y.filter(|value| value.is_finite()) else {
            self.reset();
            return ScrollUpdate::default();
        };
        if !two_fingers_extended {
            self.reset();
            return ScrollUpdate::default();
        }

        let y = y.clamp(0.0, 1.0);
        let Some(previous_y) = self.previous_y.replace(y) else {
            return ScrollUpdate {
                active: true,
                wheel_delta: 0,
            };
        };

        self.pending_motion += previous_y - y;
        if self.pending_motion.abs() < MINIMUM_SCROLL_MOTION {
            return ScrollUpdate {
                active: true,
                wheel_delta: 0,
            };
        }

        let sensitivity = if sensitivity.is_finite() {
            sensitivity.clamp(0.4, 3.0)
        } else {
            1.0
        };
        let direction = if reversed { -1.0 } else { 1.0 };
        let raw_delta =
            (self.pending_motion * SCROLL_UNITS_PER_NORMALIZED_POINT * sensitivity * direction)
                .round();
        let wheel_delta =
            (raw_delta as i32).clamp(-MAX_WHEEL_DELTA_PER_SAMPLE, MAX_WHEEL_DELTA_PER_SAMPLE);
        self.pending_motion = 0.0;
        ScrollUpdate {
            active: true,
            wheel_delta,
        }
    }

    pub fn reset(&mut self) {
        self.previous_y = None;
        self.pending_motion = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollRecognizer;

    #[test]
    fn upward_hand_motion_scrolls_up_and_downward_motion_scrolls_down() {
        let mut recognizer = ScrollRecognizer::default();
        assert_eq!(
            recognizer.update(true, Some(0.5), 1.0, false).wheel_delta,
            0
        );
        assert!(recognizer.update(true, Some(0.48), 1.0, false).wheel_delta > 0);
        assert!(recognizer.update(true, Some(0.52), 1.0, false).wheel_delta < 0);
    }

    #[test]
    fn reverse_direction_and_sensitivity_apply_to_scroll_delta() {
        let mut normal = ScrollRecognizer::default();
        normal.update(true, Some(0.6), 1.0, false);
        let normal_delta = normal.update(true, Some(0.58), 1.0, false).wheel_delta;

        let mut reversed = ScrollRecognizer::default();
        reversed.update(true, Some(0.6), 2.0, true);
        let reversed_delta = reversed.update(true, Some(0.58), 2.0, true).wheel_delta;
        assert_eq!(reversed_delta, -normal_delta * 2);
    }

    #[test]
    fn small_motion_is_ignored_and_tracking_loss_clears_scroll_state() {
        let mut recognizer = ScrollRecognizer::default();
        recognizer.update(true, Some(0.5), 1.0, false);
        assert_eq!(
            recognizer.update(true, Some(0.499), 1.0, false).wheel_delta,
            0
        );
        let lost = recognizer.update(false, None, 1.0, false);
        assert!(!lost.active);
        assert_eq!(
            recognizer.update(true, Some(0.4), 1.0, false).wheel_delta,
            0
        );
    }

    #[test]
    fn scroll_spikes_are_bounded_per_frame() {
        let mut recognizer = ScrollRecognizer::default();
        recognizer.update(true, Some(1.0), 3.0, false);
        assert_eq!(
            recognizer.update(true, Some(0.0), 3.0, false).wheel_delta,
            480
        );
    }
}
