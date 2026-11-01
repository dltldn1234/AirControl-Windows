use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorCalibration {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Default for CursorCalibration {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        }
    }
}

impl CursorCalibration {
    pub fn normalized(self) -> Self {
        const MINIMUM_SPAN: f32 = 0.15;
        let valid = [self.left, self.top, self.right, self.bottom]
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
            && self.right - self.left >= MINIMUM_SPAN
            && self.bottom - self.top >= MINIMUM_SPAN;
        if valid { self } else { Self::default() }
    }
}

pub fn apply_cursor_calibration(
    point: [f32; 2],
    calibration: CursorCalibration,
) -> Option<[f32; 2]> {
    if point.iter().any(|value| !value.is_finite()) {
        return None;
    }
    let calibration = calibration.normalized();
    Some([
        ((point[0] - calibration.left) / (calibration.right - calibration.left)).clamp(0.0, 1.0),
        ((point[1] - calibration.top) / (calibration.bottom - calibration.top)).clamp(0.0, 1.0),
    ])
}

/// Pixel bounds of the complete Windows virtual desktop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VirtualDesktop {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

/// A physical desktop pixel coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

/// Applies a bounded movement gain around the camera center. Values above one
/// use a larger portion of the display for the same hand movement.
pub fn apply_cursor_sensitivity(point: [f32; 2], sensitivity: f32) -> Option<[f32; 2]> {
    if !point[0].is_finite() || !point[1].is_finite() {
        return None;
    }
    let gain = if sensitivity.is_finite() {
        sensitivity.clamp(0.5, 2.0)
    } else {
        1.0
    };
    Some([
        ((point[0] - 0.5) * gain + 0.5).clamp(0.0, 1.0),
        ((point[1] - 0.5) * gain + 0.5).clamp(0.0, 1.0),
    ])
}

/// Maps a camera-normalized point to the virtual desktop, optionally mirroring
/// the horizontal camera axis to match a mirrored preview.
pub fn map_normalized_cursor(
    point: [f32; 2],
    desktop: VirtualDesktop,
    mirrored: bool,
) -> Option<ScreenPoint> {
    if desktop.width == 0 || desktop.height == 0 || !point[0].is_finite() || !point[1].is_finite() {
        return None;
    }

    let normalized_x = if mirrored { 1.0 - point[0] } else { point[0] }.clamp(0.0, 1.0);
    let normalized_y = point[1].clamp(0.0, 1.0);
    let x = i64::from(desktop.left)
        + (f64::from(normalized_x) * f64::from(desktop.width - 1)).round() as i64;
    let y = i64::from(desktop.top)
        + (f64::from(normalized_y) * f64::from(desktop.height - 1)).round() as i64;
    Some(ScreenPoint {
        x: i32::try_from(x).ok()?,
        y: i32::try_from(y).ok()?,
    })
}

/// Converts a pixel in the virtual desktop to the 0..=65535 coordinates used
/// by absolute SendInput mouse motion.
pub fn normalize_absolute_cursor(point: ScreenPoint, desktop: VirtualDesktop) -> Option<[i32; 2]> {
    if desktop.width == 0
        || desktop.height == 0
        || desktop.width > i32::MAX as u32
        || desktop.height > i32::MAX as u32
    {
        return None;
    }

    let x = point
        .x
        .saturating_sub(desktop.left)
        .clamp(0, desktop.width as i32 - 1);
    let y = point
        .y
        .saturating_sub(desktop.top)
        .clamp(0, desktop.height as i32 - 1);
    let max_x = desktop.width.saturating_sub(1);
    let max_y = desktop.height.saturating_sub(1);

    Some([
        normalize_axis(x as u32, max_x),
        normalize_axis(y as u32, max_y),
    ])
}

fn normalize_axis(value: u32, maximum: u32) -> i32 {
    if maximum == 0 {
        0
    } else {
        ((u64::from(value) * 65_535 + u64::from(maximum) / 2) / u64::from(maximum)) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CursorCalibration, ScreenPoint, VirtualDesktop, apply_cursor_calibration,
        apply_cursor_sensitivity, map_normalized_cursor, normalize_absolute_cursor,
    };

    const DESKTOP: VirtualDesktop = VirtualDesktop {
        left: -1920,
        top: -120,
        width: 4480,
        height: 1560,
    };

    #[test]
    fn normalized_points_cover_the_virtual_desktop_including_negative_origins() {
        assert_eq!(
            map_normalized_cursor([0.0, 0.0], DESKTOP, false),
            Some(ScreenPoint { x: -1920, y: -120 })
        );
        assert_eq!(
            map_normalized_cursor([1.0, 1.0], DESKTOP, false),
            Some(ScreenPoint { x: 2559, y: 1439 })
        );
    }

    #[test]
    fn camera_mirroring_reverses_only_the_horizontal_axis() {
        assert_eq!(
            map_normalized_cursor([0.0, 0.25], DESKTOP, true),
            Some(ScreenPoint { x: 2559, y: 270 })
        );
    }

    #[test]
    fn normalized_points_are_clamped_and_non_finite_points_are_rejected() {
        assert_eq!(
            map_normalized_cursor([-10.0, 4.0], DESKTOP, false),
            Some(ScreenPoint { x: -1920, y: 1439 })
        );
        assert_eq!(map_normalized_cursor([f32::NAN, 0.5], DESKTOP, false), None);
        assert_eq!(
            map_normalized_cursor([0.5, f32::INFINITY], DESKTOP, false),
            None
        );
    }

    #[test]
    fn invalid_desktop_extents_do_not_overflow_or_panic() {
        let desktop = VirtualDesktop {
            left: i32::MAX,
            top: 0,
            width: 10,
            height: 10,
        };
        assert_eq!(map_normalized_cursor([1.0, 0.5], desktop, false), None);
        assert_eq!(
            normalize_absolute_cursor(
                ScreenPoint { x: 0, y: 0 },
                VirtualDesktop {
                    width: u32::MAX,
                    ..desktop
                }
            ),
            None
        );
    }

    #[test]
    fn absolute_input_coordinates_respect_negative_origins_and_boundaries() {
        assert_eq!(
            normalize_absolute_cursor(ScreenPoint { x: -1920, y: -120 }, DESKTOP),
            Some([0, 0])
        );
        assert_eq!(
            normalize_absolute_cursor(ScreenPoint { x: 2559, y: 1439 }, DESKTOP),
            Some([65_535, 65_535])
        );
        assert_eq!(
            normalize_absolute_cursor(
                ScreenPoint {
                    x: i32::MAX,
                    y: i32::MIN
                },
                DESKTOP
            ),
            Some([65_535, 0])
        );
    }

    #[test]
    fn one_pixel_desktops_map_to_the_only_valid_pixel_and_input_origin() {
        let desktop = VirtualDesktop {
            left: -5,
            top: 8,
            width: 1,
            height: 1,
        };
        assert_eq!(
            map_normalized_cursor([0.9, 0.2], desktop, false),
            Some(ScreenPoint { x: -5, y: 8 })
        );
        assert_eq!(
            normalize_absolute_cursor(ScreenPoint { x: -5, y: 8 }, desktop),
            Some([0, 0])
        );
    }

    #[test]
    fn sensitivity_scales_around_center_and_clamps_at_display_edges() {
        assert_eq!(apply_cursor_sensitivity([0.5, 0.5], 2.0), Some([0.5, 0.5]));
        assert_eq!(
            apply_cursor_sensitivity([0.75, 0.25], 2.0),
            Some([1.0, 0.0])
        );
        assert_eq!(
            apply_cursor_sensitivity([0.25, 0.75], 0.5),
            Some([0.375, 0.625])
        );
    }

    #[test]
    fn calibration_maps_user_range_to_full_display_and_clamps_outside_points() {
        let calibration = CursorCalibration {
            left: 0.2,
            top: 0.1,
            right: 0.8,
            bottom: 0.9,
        };
        assert_eq!(
            apply_cursor_calibration([0.2, 0.1], calibration),
            Some([0.0, 0.0])
        );
        let center = apply_cursor_calibration([0.5, 0.5], calibration).unwrap();
        assert!((center[0] - 0.5).abs() < f32::EPSILON);
        assert!((center[1] - 0.5).abs() < f32::EPSILON);
        assert_eq!(
            apply_cursor_calibration([0.8, 0.9], calibration),
            Some([1.0, 1.0])
        );
        assert_eq!(
            apply_cursor_calibration([0.0, 1.0], calibration),
            Some([0.0, 1.0])
        );
    }

    #[test]
    fn invalid_calibration_falls_back_to_full_camera_frame() {
        let invalid = CursorCalibration {
            left: 0.7,
            top: 0.0,
            right: 0.6,
            bottom: 0.9,
        };
        assert_eq!(invalid.normalized(), CursorCalibration::default());
        assert_eq!(
            apply_cursor_calibration([0.3, 0.6], invalid),
            Some([0.3, 0.6])
        );
        assert_eq!(
            apply_cursor_calibration([f32::NAN, 0.5], CursorCalibration::default()),
            None
        );
    }

    #[test]
    fn calibrated_edges_compose_with_mirroring_and_virtual_desktop_mapping() {
        let calibration = CursorCalibration {
            left: 0.2,
            top: 0.1,
            right: 0.8,
            bottom: 0.9,
        };
        let camera_left = apply_cursor_calibration([0.2, 0.1], calibration).unwrap();
        let camera_right = apply_cursor_calibration([0.8, 0.9], calibration).unwrap();

        assert_eq!(
            map_normalized_cursor(camera_left, DESKTOP, true),
            Some(ScreenPoint { x: 2559, y: -120 })
        );
        assert_eq!(
            map_normalized_cursor(camera_right, DESKTOP, true),
            Some(ScreenPoint { x: -1920, y: 1439 })
        );
    }
}
