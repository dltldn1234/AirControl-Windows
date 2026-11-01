use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowSizeLimits {
    pub minimum: [u32; 2],
    pub maximum: [u32; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowCandidate<'a> {
    pub handle: u64,
    pub title: &'a str,
    pub process_id: u32,
    pub visible: bool,
    pub owned: bool,
    pub cloaked: bool,
    pub tool_window: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowMoveUpdate {
    pub active: bool,
    pub normalized_delta: Option<[f32; 2]>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WindowMoveRecognizer {
    active: bool,
    previous_position: Option<[f32; 2]>,
}

impl WindowMoveRecognizer {
    pub fn is_active(self) -> bool {
        self.active
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.previous_position = None;
    }

    pub fn update(
        &mut self,
        pinch_distance: Option<f32>,
        position: Option<[f32; 2]>,
        sensitivity: f32,
    ) -> WindowMoveUpdate {
        let Some(distance) = pinch_distance.filter(|value| value.is_finite()) else {
            self.reset();
            return WindowMoveUpdate::default();
        };
        let Some(position) = position.filter(|point| point.iter().all(|value| value.is_finite()))
        else {
            self.reset();
            return WindowMoveUpdate::default();
        };
        let sensitivity = if sensitivity.is_finite() {
            sensitivity.clamp(0.6, 1.4)
        } else {
            1.0
        };
        let start_threshold = 0.28 * sensitivity;
        let release_threshold = 0.42 * sensitivity;

        if self.active {
            if distance >= release_threshold {
                self.reset();
                return WindowMoveUpdate::default();
            }
            let normalized_delta = self
                .previous_position
                .replace(position)
                .map(|previous| [position[0] - previous[0], position[1] - previous[1]]);
            return WindowMoveUpdate {
                active: true,
                normalized_delta,
            };
        }
        if distance <= start_threshold {
            self.active = true;
            self.previous_position = Some(position);
            return WindowMoveUpdate {
                active: true,
                normalized_delta: None,
            };
        }
        WindowMoveUpdate::default()
    }
}

pub fn is_supported_window(candidate: WindowCandidate<'_>, current_process_id: u32) -> bool {
    candidate.handle != 0
        && !candidate.title.trim().is_empty()
        && candidate.process_id != 0
        && candidate.process_id != current_process_id
        && candidate.visible
        && !candidate.owned
        && !candidate.cloaked
        && !candidate.tool_window
}

pub fn move_window(rect: WindowRect, delta: [f32; 2]) -> Option<WindowRect> {
    if rect.width == 0 || rect.height == 0 || delta.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(WindowRect {
        x: rect.x.saturating_add(delta[0].round() as i32),
        y: rect.y.saturating_add(delta[1].round() as i32),
        ..rect
    })
}

pub fn resize_window(rect: WindowRect, scale: f32, limits: WindowSizeLimits) -> Option<WindowRect> {
    if rect.width == 0
        || rect.height == 0
        || !scale.is_finite()
        || scale <= 0.0
        || limits.minimum.contains(&0)
        || limits.minimum[0] > limits.maximum[0]
        || limits.minimum[1] > limits.maximum[1]
    {
        return None;
    }

    let width = (rect.width as f32 * scale)
        .round()
        .clamp(limits.minimum[0] as f32, limits.maximum[0] as f32) as u32;
    let height = (rect.height as f32 * scale)
        .round()
        .clamp(limits.minimum[1] as f32, limits.maximum[1] as f32) as u32;
    let center_x = i64::from(rect.x) + i64::from(rect.width) / 2;
    let center_y = i64::from(rect.y) + i64::from(rect.height) / 2;

    Some(WindowRect {
        x: (center_x - i64::from(width) / 2).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        y: (center_y - i64::from(height) / 2).clamp(i64::from(i32::MIN), i64::from(i32::MAX))
            as i32,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        WindowCandidate, WindowMoveRecognizer, WindowRect, WindowSizeLimits, is_supported_window,
        move_window, resize_window,
    };

    #[test]
    fn moves_window_with_fractional_and_negative_deltas() {
        let moved = move_window(
            WindowRect {
                x: -1280,
                y: 40,
                width: 800,
                height: 600,
            },
            [15.6, -20.4],
        )
        .expect("valid move");
        assert_eq!(moved.x, -1264);
        assert_eq!(moved.y, 20);
        assert_eq!([moved.width, moved.height], [800, 600]);
    }

    #[test]
    fn rejects_invalid_rectangles_and_non_finite_move_deltas() {
        let rect = WindowRect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        assert!(move_window(rect, [f32::NAN, 0.0]).is_none());
        assert!(move_window(WindowRect { width: 0, ..rect }, [1.0, 1.0]).is_none());
    }

    #[test]
    fn resizes_around_center_and_respects_minimum_and_maximum_limits() {
        let rect = WindowRect {
            x: -100,
            y: 200,
            width: 600,
            height: 400,
        };
        let enlarged = resize_window(
            rect,
            3.0,
            WindowSizeLimits {
                minimum: [320, 240],
                maximum: [1200, 900],
            },
        )
        .expect("valid resize");
        assert_eq!([enlarged.width, enlarged.height], [1200, 900]);
        assert_eq!(i64::from(enlarged.x) + 600, i64::from(rect.x) + 300);
        assert_eq!(i64::from(enlarged.y) + 450, i64::from(rect.y) + 200);

        let reduced = resize_window(
            rect,
            0.1,
            WindowSizeLimits {
                minimum: [320, 240],
                maximum: [1200, 900],
            },
        )
        .expect("minimum resize");
        assert_eq!([reduced.width, reduced.height], [320, 240]);
    }

    #[test]
    fn rejects_invalid_resize_limits() {
        assert!(
            resize_window(
                WindowRect {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                2.0,
                WindowSizeLimits {
                    minimum: [800, 100],
                    maximum: [400, 900],
                }
            )
            .is_none()
        );
    }

    #[test]
    fn excludes_hidden_owned_cloaked_tool_and_self_windows() {
        let candidate = WindowCandidate {
            handle: 2,
            title: "Example",
            process_id: 42,
            visible: true,
            owned: false,
            cloaked: false,
            tool_window: false,
        };
        assert!(is_supported_window(candidate, 7));
        assert!(!is_supported_window(candidate, 42));
        assert!(!is_supported_window(
            WindowCandidate {
                cloaked: true,
                ..candidate
            },
            7
        ));
        assert!(!is_supported_window(
            WindowCandidate {
                title: " ",
                ..candidate
            },
            7
        ));
    }

    #[test]
    fn window_move_pinch_uses_hysteresis_and_emits_deltas_while_held() {
        let mut recognizer = WindowMoveRecognizer::default();
        let start = recognizer.update(Some(0.2), Some([0.3, 0.4]), 1.0);
        assert!(start.active);
        assert_eq!(start.normalized_delta, None);

        let held = recognizer.update(Some(0.35), Some([0.4, 0.45]), 1.0);
        assert!(held.active);
        let delta = held
            .normalized_delta
            .expect("active pinch reports movement");
        assert!((delta[0] - 0.1).abs() < 1e-6);
        assert!((delta[1] - 0.05).abs() < 1e-6);

        let ended = recognizer.update(Some(0.5), Some([0.5, 0.5]), 1.0);
        assert!(!ended.active);
        assert_eq!(ended.normalized_delta, None);
    }

    #[test]
    fn tracking_loss_ends_window_move_without_a_final_delta() {
        let mut recognizer = WindowMoveRecognizer::default();
        recognizer.update(Some(0.1), Some([0.5, 0.5]), 1.0);
        assert!(!recognizer.update(None, None, 1.0).active);
        assert!(!recognizer.is_active());
        assert!(
            !recognizer
                .update(Some(0.1), Some([f32::NAN, 0.2]), 1.0)
                .active
        );
    }
}
