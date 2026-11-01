use std::time::Instant;

const DERIVATIVE_CUTOFF: f32 = 1.0;
const DEAD_ZONE: f32 = 0.0015;

#[derive(Clone, Copy, Debug, Default)]
pub struct OneEuroCursorFilter {
    sample: Option<FilterSample>,
}

#[derive(Clone, Copy, Debug)]
struct FilterSample {
    raw: [f32; 2],
    filtered: [f32; 2],
    derivative: [f32; 2],
    at: Instant,
}

impl OneEuroCursorFilter {
    pub fn update(&mut self, point: [f32; 2], smoothing: f32, now: Instant) -> Option<[f32; 2]> {
        if point.iter().any(|coordinate| !coordinate.is_finite()) {
            return None;
        }
        let smoothing = if smoothing.is_finite() {
            smoothing.clamp(0.0, 0.9)
        } else {
            0.4
        };
        let Some(previous) = self.sample else {
            self.sample = Some(FilterSample {
                raw: point,
                filtered: point,
                derivative: [0.0; 2],
                at: now,
            });
            return Some(point);
        };

        let delta = now
            .saturating_duration_since(previous.at)
            .as_secs_f32()
            .max(0.001);
        if distance(point, previous.filtered) <= DEAD_ZONE {
            self.sample = Some(FilterSample {
                at: now,
                ..previous
            });
            return Some(previous.filtered);
        }

        let min_cutoff = 1.5 - smoothing * 1.2;
        let signal_alpha = |cutoff| alpha(cutoff, delta);
        let derivative_alpha = signal_alpha(DERIVATIVE_CUTOFF);
        let raw_derivative = [
            (point[0] - previous.raw[0]) / delta,
            (point[1] - previous.raw[1]) / delta,
        ];
        let derivative = low_pass(previous.derivative, raw_derivative, derivative_alpha);
        let cutoff = [
            min_cutoff + 0.28 * derivative[0].abs(),
            min_cutoff + 0.28 * derivative[1].abs(),
        ];
        let filtered = [
            low_pass_scalar(previous.filtered[0], point[0], signal_alpha(cutoff[0])),
            low_pass_scalar(previous.filtered[1], point[1], signal_alpha(cutoff[1])),
        ];
        self.sample = Some(FilterSample {
            raw: point,
            filtered,
            derivative,
            at: now,
        });
        Some(filtered)
    }

    pub fn reset(&mut self) {
        self.sample = None;
    }
}

fn alpha(cutoff: f32, delta_seconds: f32) -> f32 {
    let tau = 1.0 / (std::f32::consts::TAU * cutoff.max(f32::EPSILON));
    1.0 / (1.0 + tau / delta_seconds)
}

fn low_pass(previous: [f32; 2], current: [f32; 2], coefficient: f32) -> [f32; 2] {
    [
        low_pass_scalar(previous[0], current[0], coefficient),
        low_pass_scalar(previous[1], current[1], coefficient),
    ]
}

fn low_pass_scalar(previous: f32, current: f32, coefficient: f32) -> f32 {
    coefficient * current + (1.0 - coefficient) * previous
}

fn distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
}

#[cfg(test)]
mod tests {
    use super::OneEuroCursorFilter;
    use std::time::{Duration, Instant};

    #[test]
    fn first_valid_sample_is_unfiltered_and_non_finite_input_is_rejected() {
        let mut filter = OneEuroCursorFilter::default();
        let now = Instant::now();
        assert_eq!(filter.update([0.25, 0.75], 0.4, now), Some([0.25, 0.75]));
        assert_eq!(filter.update([f32::NAN, 0.5], 0.4, now), None);
    }

    #[test]
    fn dead_zone_holds_micro_jitter_and_fast_motion_gets_a_higher_cutoff() {
        let now = Instant::now();
        let mut filter = OneEuroCursorFilter::default();
        let initial = filter.update([0.5, 0.5], 0.5, now).expect("finite sample");
        assert_eq!(
            filter.update([0.5005, 0.5005], 0.5, now + Duration::from_millis(100)),
            Some(initial)
        );
        let slow = filter
            .update([0.52, 0.52], 0.8, now + Duration::from_millis(200))
            .expect("finite sample");

        let mut responsive_filter = OneEuroCursorFilter::default();
        responsive_filter.update([0.5, 0.5], 0.1, now);
        let responsive = responsive_filter
            .update([0.9, 0.9], 0.1, now + Duration::from_millis(100))
            .expect("finite sample");
        assert!(slow[0] > initial[0]);
        assert!(responsive[0] > 0.5);
        assert!(responsive[0] < 0.9);
    }

    #[test]
    fn reset_makes_the_next_position_immediate() {
        let now = Instant::now();
        let mut filter = OneEuroCursorFilter::default();
        filter.update([0.1, 0.1], 0.8, now);
        filter.update([0.8, 0.8], 0.8, now + Duration::from_millis(10));
        filter.reset();
        assert_eq!(
            filter.update([0.9, 0.2], 0.8, now + Duration::from_secs(1)),
            Some([0.9, 0.2])
        );
    }
}
