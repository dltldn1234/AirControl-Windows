use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SwipeDirection {
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug)]
struct SwipeStart {
    point: [f32; 2],
    at: Instant,
}

/// Recognizes deliberate horizontal hand swipes for cycling eligible apps.
/// Pass `None` when tracking is lost so a partial swipe cannot resume later.
#[derive(Clone, Copy, Debug, Default)]
pub struct AppSwitchSwipeRecognizer {
    start: Option<SwipeStart>,
    last_emitted_at: Option<Instant>,
}

impl AppSwitchSwipeRecognizer {
    const MINIMUM_DISTANCE: f32 = 0.24;
    const MAXIMUM_VERTICAL_DRIFT: f32 = 0.16;
    const MINIMUM_DURATION: Duration = Duration::from_millis(80);
    const MAXIMUM_DURATION: Duration = Duration::from_millis(700);
    const COOLDOWN: Duration = Duration::from_millis(900);
    const MINIMUM_VELOCITY: f32 = 0.38;

    pub fn update(&mut self, point: Option<[f32; 2]>, now: Instant) -> Option<SwipeDirection> {
        let Some(point) = point.filter(|point| {
            point
                .iter()
                .all(|coordinate| coordinate.is_finite() && (0.0..=1.0).contains(coordinate))
        }) else {
            self.start = None;
            return None;
        };

        let Some(start) = self.start else {
            self.start = Some(SwipeStart { point, at: now });
            return None;
        };

        let elapsed = now.saturating_duration_since(start.at);
        let delta_x = point[0] - start.point[0];
        let delta_y = (point[1] - start.point[1]).abs();
        if elapsed > Self::MAXIMUM_DURATION {
            self.start = Some(SwipeStart { point, at: now });
            return None;
        }

        if elapsed < Self::MINIMUM_DURATION
            || delta_x.abs() < Self::MINIMUM_DISTANCE
            || delta_y > Self::MAXIMUM_VERTICAL_DRIFT
            || delta_x.abs() / elapsed.as_secs_f32().max(f32::EPSILON) < Self::MINIMUM_VELOCITY
        {
            return None;
        }

        self.start = None;
        let cooled_down = self
            .last_emitted_at
            .is_none_or(|last| now.saturating_duration_since(last) >= Self::COOLDOWN);
        if !cooled_down {
            return None;
        }
        self.last_emitted_at = Some(now);
        Some(if delta_x > 0.0 {
            SwipeDirection::Next
        } else {
            SwipeDirection::Previous
        })
    }

    pub fn reset(&mut self) {
        self.start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{AppSwitchSwipeRecognizer, SwipeDirection};
    use std::time::{Duration, Instant};

    #[test]
    fn recognizes_deliberate_swipes_in_both_directions() {
        let start = Instant::now();
        let mut recognizer = AppSwitchSwipeRecognizer::default();
        assert_eq!(recognizer.update(Some([0.25, 0.5]), start), None);
        assert_eq!(
            recognizer.update(Some([0.52, 0.53]), start + Duration::from_millis(240)),
            Some(SwipeDirection::Next)
        );

        assert_eq!(
            recognizer.update(Some([0.75, 0.5]), start + Duration::from_secs(2)),
            None
        );
        assert_eq!(
            recognizer.update(Some([0.45, 0.52]), start + Duration::from_millis(2_240)),
            Some(SwipeDirection::Previous)
        );
    }

    #[test]
    fn rejects_slow_vertical_and_out_of_bounds_motion() {
        let start = Instant::now();
        let mut recognizer = AppSwitchSwipeRecognizer::default();
        recognizer.update(Some([0.2, 0.5]), start);
        assert_eq!(
            recognizer.update(Some([0.5, 0.51]), start + Duration::from_millis(900)),
            None
        );

        recognizer.update(Some([0.2, 0.5]), start + Duration::from_secs(2));
        assert_eq!(
            recognizer.update(Some([0.5, 0.8]), start + Duration::from_millis(2_240)),
            None
        );
        assert_eq!(recognizer.update(Some([1.2, 0.5]), start), None);
    }

    #[test]
    fn cooldown_and_tracking_loss_prevent_repeated_or_resumed_swipes() {
        let start = Instant::now();
        let mut recognizer = AppSwitchSwipeRecognizer::default();
        recognizer.update(Some([0.2, 0.5]), start);
        assert_eq!(
            recognizer.update(Some([0.5, 0.5]), start + Duration::from_millis(200)),
            Some(SwipeDirection::Next)
        );

        recognizer.update(Some([0.2, 0.5]), start + Duration::from_millis(300));
        assert_eq!(
            recognizer.update(Some([0.5, 0.5]), start + Duration::from_millis(500)),
            None
        );

        recognizer.update(Some([0.2, 0.5]), start + Duration::from_secs(2));
        assert_eq!(
            recognizer.update(None, start + Duration::from_millis(2_100)),
            None
        );
        assert_eq!(
            recognizer.update(Some([0.5, 0.5]), start + Duration::from_millis(2_200)),
            None
        );
    }
}
