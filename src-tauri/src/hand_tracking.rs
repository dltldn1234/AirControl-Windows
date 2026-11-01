use ort::environment::Environment;
use ort::session::{Session, builder::GraphOptimizationLevel};
use std::sync::{Arc, OnceLock};

const PALM_MODEL: &[u8] = include_bytes!("../models/palm_detection_mediapipe_2023feb.onnx");
const LANDMARK_MODEL: &[u8] =
    include_bytes!("../models/handpose_estimation_mediapipe_2023feb.onnx");

static ENVIRONMENT: OnceLock<Result<Arc<Environment>, String>> = OnceLock::new();

pub struct HandTrackingModels {
    _palm_detection: Session,
    _hand_landmarks: Session,
}

impl HandTrackingModels {
    pub fn load() -> Result<Self, String> {
        let environment = ENVIRONMENT
            .get_or_init(|| {
                ort::init()
                    .with_name("AirControl")
                    .with_telemetry(false)
                    .build()
                    .map(Arc::new)
                    .map_err(|error| format!("ONNX Runtime could not start: {error}"))
            })
            .as_ref()
            .map_err(Clone::clone)?;

        let palm_detection = load_session(environment, PALM_MODEL, "palm detection")?;
        let hand_landmarks = load_session(environment, LANDMARK_MODEL, "hand landmarks")?;
        Ok(Self {
            _palm_detection: palm_detection,
            _hand_landmarks: hand_landmarks,
        })
    }
}

fn load_session(environment: &Environment, model: &[u8], name: &str) -> Result<Session, String> {
    let builder = Session::builder(environment)
        .map_err(|error| format!("Could not configure {name} model: {error}"))?;
    let builder = builder
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|error| format!("Could not optimize {name} model: {error}"))?;
    let mut builder = builder
        .with_intra_threads(2)
        .map_err(|error| format!("Could not configure {name} model threads: {error}"))?;
    builder
        .commit_from_memory(model)
        .map_err(|error| format!("Could not load {name} model: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{HandTrackingModels, LANDMARK_MODEL, PALM_MODEL};

    #[test]
    fn bundled_models_are_present_and_distinct() {
        assert!(PALM_MODEL.starts_with(b"\x08\x07"));
        assert!(LANDMARK_MODEL.starts_with(b"\x08\x07"));
        assert_ne!(PALM_MODEL, LANDMARK_MODEL);
    }

    #[test]
    fn bundled_onnx_models_load_into_runtime_sessions() {
        let models = HandTrackingModels::load().expect("bundled ONNX models should load");

        assert_eq!(models._palm_detection.inputs().len(), 1);
        assert_eq!(models._hand_landmarks.inputs().len(), 1);
        assert_eq!(models._palm_detection.outputs().len(), 2);
        assert_eq!(models._hand_landmarks.outputs().len(), 4);
    }
}
