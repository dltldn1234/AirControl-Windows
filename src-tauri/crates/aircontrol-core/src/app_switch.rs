use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SwipeDirection {
    Previous,
    Next,
}

pub fn next_window_index(
    current: Option<usize>,
    window_count: usize,
    direction: SwipeDirection,
) -> Option<usize> {
    if window_count == 0 {
        return None;
    }
    Some(
        match (direction, current.filter(|index| *index < window_count)) {
            (SwipeDirection::Next, Some(index)) => (index + 1) % window_count,
            (SwipeDirection::Previous, Some(0)) => window_count - 1,
            (SwipeDirection::Previous, Some(index)) => index - 1,
            (SwipeDirection::Next, None) => 0,
            (SwipeDirection::Previous, None) => window_count - 1,
        },
    )
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

pub fn three_finger_swipe_position(
    landmarks: &[[f32; 2]; 21],
    frame_size: [u32; 2],
    mirrored: bool,
) -> Option<[f32; 2]> {
    if frame_size.contains(&0) || landmarks.iter().flatten().any(|value| !value.is_finite()) {
        return None;
    }

    let index = finger_is_extended(landmarks, 5, 6, 8);
    let middle = finger_is_extended(landmarks, 9, 10, 12);
    let ring = finger_is_extended(landmarks, 13, 14, 16);
    let pinky = finger_is_extended(landmarks, 17, 18, 20);
    if !(index && middle && ring && !pinky) {
        return None;
    }

    let x = (landmarks[8][0] + landmarks[12][0] + landmarks[16][0]) / (3.0 * frame_size[0] as f32);
    let y = (landmarks[8][1] + landmarks[12][1] + landmarks[16][1]) / (3.0 * frame_size[1] as f32);
    Some([if mirrored { 1.0 - x } else { x }, y])
}

fn finger_is_extended(landmarks: &[[f32; 2]; 21], mcp: usize, pip: usize, tip: usize) -> bool {
    let proximal = distance(landmarks[mcp], landmarks[pip]);
    proximal > f32::EPSILON && distance(landmarks[mcp], landmarks[tip]) >= proximal * 1.45
}

fn distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
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

    #[test]
    fn swipe_pointer_requires_three_extended_fingers_and_applies_mirroring() {
        use super::three_finger_swipe_position;

        let mut landmarks = [[0.0, 0.0]; 21];
        for (mcp, pip, tip, x) in [(5, 6, 8, 50.0), (9, 10, 12, 70.0), (13, 14, 16, 90.0)] {
            landmarks[mcp] = [x, 100.0];
            landmarks[pip] = [x, 80.0];
            landmarks[tip] = [x, 40.0];
        }
        landmarks[17] = [100.0, 100.0];
        landmarks[18] = [100.0, 80.0];
        landmarks[20] = [100.0, 75.0];

        assert_eq!(
            three_finger_swipe_position(&landmarks, [200, 200], false),
            Some([0.35, 0.2])
        );
        assert_eq!(
            three_finger_swipe_position(&landmarks, [200, 200], true),
            Some([0.65, 0.2])
        );

        landmarks[20] = [100.0, 30.0];
        assert_eq!(
            three_finger_swipe_position(&landmarks, [200, 200], false),
            None
        );
        assert_eq!(
            three_finger_swipe_position(&landmarks, [0, 200], false),
            None
        );
    }

    #[test]
    fn application_cycle_wraps_in_both_directions_and_handles_missing_windows() {
        use super::next_window_index;

        assert_eq!(
            next_window_index(Some(0), 3, SwipeDirection::Previous),
            Some(2)
        );
        assert_eq!(next_window_index(Some(2), 3, SwipeDirection::Next), Some(0));
        assert_eq!(next_window_index(None, 3, SwipeDirection::Next), Some(0));
        assert_eq!(
            next_window_index(None, 3, SwipeDirection::Previous),
            Some(2)
        );
        assert_eq!(next_window_index(Some(8), 3, SwipeDirection::Next), Some(0));
        assert_eq!(next_window_index(Some(0), 0, SwipeDirection::Next), None);
    }
}
