const TRACKED_HAND_COUNT: usize = 2;
const MAX_MATCH_DISTANCE: f32 = 0.35;
const MAX_MISSED_FRAMES: u8 = 2;

#[derive(Clone, Copy, Debug)]
struct HandTrack {
    center: [f32; 2],
    velocity: [f32; 2],
    missed_frames: u8,
}

/// Associates confidence-ordered hand detections with short-lived spatial tracks.
/// The returned indices preserve identity order; missing hands stay `None` so a
/// consumer cannot accidentally reuse landmarks from a previous frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct HandTrackAssociator {
    tracks: [Option<HandTrack>; TRACKED_HAND_COUNT],
}

impl HandTrackAssociator {
    /// Returns the current detection index for each stable track slot.
    /// Centers must be normalized to the camera frame (`0.0..=1.0`).
    pub fn update(&mut self, centers: &[Option<[f32; 2]>]) -> [Option<usize>; TRACKED_HAND_COUNT] {
        let candidates = candidates(centers);
        let choices = [Some(0), Some(1), None];
        let mut best_assignment = [None, None];
        let mut best_matches = 0usize;
        let mut best_cost = f32::INFINITY;
        let mut found_assignment = false;

        for first in choices {
            for second in choices {
                if first.is_some() && first == second {
                    continue;
                }
                let assignment = [first, second];
                if assignment
                    .iter()
                    .flatten()
                    .any(|candidate_index| candidates[*candidate_index].is_none())
                {
                    continue;
                }
                let mut matches = 0;
                let mut cost = 0.0;
                let mut valid = true;
                for (slot, candidate_index) in assignment.iter().copied().enumerate() {
                    let Some(candidate_index) = candidate_index else {
                        continue;
                    };
                    if let Some(track) = self.tracks[slot]
                        && track.missed_frames <= MAX_MISSED_FRAMES
                    {
                        let Some(candidate) = candidates[candidate_index] else {
                            valid = false;
                            break;
                        };
                        let distance = distance(predicted_center(track), candidate.center);
                        if distance > MAX_MATCH_DISTANCE {
                            valid = false;
                            break;
                        }
                        matches += 1;
                        cost += distance;
                    }
                }
                if valid
                    && (!found_assignment
                        || matches > best_matches
                        || (matches == best_matches && cost < best_cost))
                {
                    best_assignment = assignment;
                    best_matches = matches;
                    best_cost = cost;
                    found_assignment = true;
                }
            }
        }

        let mut ordered = [None; TRACKED_HAND_COUNT];
        let mut next_tracks = [None; TRACKED_HAND_COUNT];
        for (slot, candidate_index) in best_assignment.iter().copied().enumerate() {
            if let Some(candidate_index) = candidate_index
                && let Some(candidate) = candidates[candidate_index]
            {
                ordered[slot] = Some(candidate.original_index);
                next_tracks[slot] = Some(match self.tracks[slot] {
                    Some(track) if track.missed_frames <= MAX_MISSED_FRAMES => {
                        let elapsed = f32::from(track.missed_frames) + 1.0;
                        let measured_velocity = [
                            (candidate.center[0] - track.center[0]) / elapsed,
                            (candidate.center[1] - track.center[1]) / elapsed,
                        ];
                        HandTrack {
                            center: candidate.center,
                            velocity: [
                                (track.velocity[0] + measured_velocity[0]) * 0.5,
                                (track.velocity[1] + measured_velocity[1]) * 0.5,
                            ],
                            missed_frames: 0,
                        }
                    }
                    _ => HandTrack {
                        center: candidate.center,
                        velocity: [0.0; 2],
                        missed_frames: 0,
                    },
                });
            } else if let Some(mut track) = self.tracks[slot]
                && track.missed_frames < MAX_MISSED_FRAMES
            {
                track.missed_frames += 1;
                next_tracks[slot] = Some(track);
            }
        }
        self.tracks = next_tracks;
        ordered
    }

    pub fn reset(&mut self) {
        self.tracks = [None; TRACKED_HAND_COUNT];
    }
}

#[derive(Clone, Copy)]
struct Candidate {
    original_index: usize,
    center: [f32; 2],
}

fn candidates(centers: &[Option<[f32; 2]>]) -> [Option<Candidate>; TRACKED_HAND_COUNT] {
    let mut result = [None; TRACKED_HAND_COUNT];
    let mut next = 0;
    for (original_index, center) in centers.iter().enumerate() {
        let Some(center) = center.filter(|center| {
            center
                .iter()
                .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        }) else {
            continue;
        };
        if next == TRACKED_HAND_COUNT {
            break;
        }
        result[next] = Some(Candidate {
            original_index,
            center,
        });
        next += 1;
    }
    result
}

fn predicted_center(track: HandTrack) -> [f32; 2] {
    let elapsed = f32::from(track.missed_frames) + 1.0;
    [
        track.center[0] + track.velocity[0] * elapsed,
        track.center[1] + track.velocity[1] * elapsed,
    ]
}

fn distance(left: [f32; 2], right: [f32; 2]) -> f32 {
    (left[0] - right[0]).hypot(left[1] - right[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_order_changes_keep_the_same_hand_in_each_track_slot() {
        let mut tracks = HandTrackAssociator::default();
        assert_eq!(
            tracks.update(&[Some([0.2, 0.5]), Some([0.8, 0.5])]),
            [Some(0), Some(1)]
        );

        assert_eq!(
            tracks.update(&[Some([0.81, 0.5]), Some([0.19, 0.5])]),
            [Some(1), Some(0)]
        );
    }

    #[test]
    fn velocity_prediction_preserves_identity_when_hands_cross() {
        let mut tracks = HandTrackAssociator::default();
        tracks.update(&[Some([0.4, 0.5]), Some([0.6, 0.5])]);
        tracks.update(&[Some([0.5, 0.5]), Some([0.5, 0.5])]);

        assert_eq!(
            tracks.update(&[Some([0.4, 0.5]), Some([0.6, 0.5])]),
            [Some(1), Some(0)]
        );
    }

    #[test]
    fn a_missing_hand_is_not_reused_and_can_reacquire_its_slot() {
        let mut tracks = HandTrackAssociator::default();
        tracks.update(&[Some([0.2, 0.5]), Some([0.8, 0.5])]);

        assert_eq!(tracks.update(&[None, Some([0.82, 0.5])]), [None, Some(1)]);
        assert_eq!(
            tracks.update(&[Some([0.21, 0.5]), Some([0.83, 0.5])]),
            [Some(0), Some(1)]
        );
    }

    #[test]
    fn stale_tracks_expire_and_far_hands_start_new_tracks() {
        let mut tracks = HandTrackAssociator::default();
        tracks.update(&[Some([0.1, 0.5])]);
        for _ in 0..3 {
            assert_eq!(tracks.update(&[]), [None, None]);
        }

        assert_eq!(tracks.update(&[Some([0.9, 0.5])]), [Some(0), None]);
    }

    #[test]
    fn a_distant_detection_does_not_inherit_an_existing_hand_identity() {
        let mut tracks = HandTrackAssociator::default();
        tracks.update(&[Some([0.1, 0.5])]);

        assert_eq!(tracks.update(&[Some([0.9, 0.5])]), [None, Some(0)]);
    }

    #[test]
    fn non_finite_and_out_of_frame_centers_are_ignored() {
        let mut tracks = HandTrackAssociator::default();
        assert_eq!(
            tracks.update(&[Some([f32::NAN, 0.5]), Some([1.2, 0.5]), Some([0.3, 0.7])]),
            [Some(2), None]
        );
    }
}
