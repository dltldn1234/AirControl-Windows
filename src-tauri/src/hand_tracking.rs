use image::{RgbImage, imageops::FilterType};
use ort::environment::Environment;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Tensor;
use serde::Serialize;
use std::sync::{Arc, OnceLock};

const PALM_MODEL: &[u8] = include_bytes!("../models/palm_detection_mediapipe_2023feb.onnx");
const LANDMARK_MODEL: &[u8] =
    include_bytes!("../models/handpose_estimation_mediapipe_2023feb.onnx");
const PALM_INPUT_SIZE: u32 = 192;
const PALM_ANCHOR_COUNT: usize = 2016;
const PALM_VALUES_PER_ANCHOR: usize = 18;
const HAND_INPUT_SIZE: u32 = 224;
const HAND_LANDMARK_COUNT: usize = 21;

static ENVIRONMENT: OnceLock<Result<Arc<Environment>, String>> = OnceLock::new();

pub struct HandTrackingModels {
    palm_detection: Session,
    hand_landmarks: Session,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PalmDetection {
    pub bounds: [f32; 4],
    pub landmarks: [[f32; 2]; 7],
    pub finger_landmarks: Option<[[f32; 2]; HAND_LANDMARK_COUNT]>,
    pub confidence: f32,
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
            palm_detection,
            hand_landmarks,
        })
    }

    pub fn detect_palms(&mut self, image: &RgbImage) -> Result<Vec<PalmDetection>, String> {
        if image.width() == 0 || image.height() == 0 {
            return Ok(Vec::new());
        }
        let mut detections = {
            let (input, transform) = preprocess_palm_input(image);
            let tensor = Tensor::from_array((
                [1, PALM_INPUT_SIZE as usize, PALM_INPUT_SIZE as usize, 3],
                input,
            ))
            .map_err(|error| format!("Could not prepare the palm model input: {error}"))?;
            let outputs = self
                .palm_detection
                .run(ort::inputs![tensor])
                .map_err(|error| format!("Palm detection inference failed: {error}"))?;
            let (_, boxes) = outputs[0]
                .try_extract_tensor::<f32>()
                .map_err(|error| format!("Palm model box output is invalid: {error}"))?;
            let (_, scores) = outputs[1]
                .try_extract_tensor::<f32>()
                .map_err(|error| format!("Palm model score output is invalid: {error}"))?;
            decode_palm_outputs(boxes, scores, transform)?
        };
        for detection in &mut detections {
            detection.finger_landmarks = self.infer_finger_landmarks(image, detection)?;
        }
        Ok(detections)
    }

    fn infer_finger_landmarks(
        &mut self,
        image: &RgbImage,
        palm: &PalmDetection,
    ) -> Result<Option<[[f32; 2]; HAND_LANDMARK_COUNT]>, String> {
        let (input, transform) = preprocess_hand_input(image, palm);
        let tensor = Tensor::from_array((
            [1, HAND_INPUT_SIZE as usize, HAND_INPUT_SIZE as usize, 3],
            input,
        ))
        .map_err(|error| format!("Could not prepare the hand landmark input: {error}"))?;
        let outputs = self
            .hand_landmarks
            .run(ort::inputs![tensor])
            .map_err(|error| format!("Hand landmark inference failed: {error}"))?;
        let (_, screen_landmarks) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("Hand landmark output is invalid: {error}"))?;
        let (_, confidence) = outputs[1]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("Hand landmark confidence output is invalid: {error}"))?;
        decode_hand_landmarks(screen_landmarks, confidence, transform)
    }
}

#[derive(Clone, Copy)]
struct HandPoseTransform {
    crop_bounds: [f32; 4],
    rotation_radians: f32,
    canvas_center: [f32; 2],
    source_offset: [f32; 2],
    source_width: u32,
    source_height: u32,
}

fn preprocess_hand_input(image: &RgbImage, palm: &PalmDetection) -> (Vec<f32>, HandPoseTransform) {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let diagonal = ((width as f32).hypot(height as f32).ceil() as u32).max(1);
    let offset_x = (diagonal - width) / 2;
    let offset_y = (diagonal - height) / 2;
    let mut padded = RgbImage::new(diagonal, diagonal);
    image::imageops::replace(&mut padded, image, i64::from(offset_x), i64::from(offset_y));

    let palm_base = [
        palm.landmarks[0][0] + offset_x as f32,
        palm.landmarks[0][1] + offset_y as f32,
    ];
    let middle_base = [
        palm.landmarks[2][0] + offset_x as f32,
        palm.landmarks[2][1] + offset_y as f32,
    ];
    let direction = (middle_base[1] - palm_base[1]).atan2(middle_base[0] - palm_base[0]);
    let rotation_radians = normalize_radians(std::f32::consts::FRAC_PI_2 + direction);
    let canvas_center = [diagonal as f32 / 2.0, diagonal as f32 / 2.0];
    let rotated = rotate_image(&padded, rotation_radians, canvas_center);
    let rotated_palm = palm.landmarks.map(|[x, y]| {
        rotate_point(
            [x + offset_x as f32, y + offset_y as f32],
            canvas_center,
            rotation_radians,
        )
    });
    let mut palm_bounds = points_bounds(&rotated_palm);
    let palm_height = palm_bounds[3] - palm_bounds[1];
    palm_bounds[1] -= palm_height * 0.4;
    palm_bounds = enlarge_bounds(palm_bounds, 3.0, diagonal);

    let crop_left = palm_bounds[0].floor().max(0.0) as u32;
    let crop_top = palm_bounds[1].floor().max(0.0) as u32;
    let crop_right = palm_bounds[2].ceil().min(diagonal as f32) as u32;
    let crop_bottom = palm_bounds[3].ceil().min(diagonal as f32) as u32;
    let crop_width = crop_right.saturating_sub(crop_left).max(1);
    let crop_height = crop_bottom.saturating_sub(crop_top).max(1);
    let square_size = crop_width.max(crop_height);
    let pad_left = (square_size - crop_width) / 2;
    let pad_top = (square_size - crop_height) / 2;
    let mut square = RgbImage::new(square_size, square_size);
    for y in crop_top..crop_bottom {
        for x in crop_left..crop_right {
            let destination_x = (x - crop_left) + pad_left;
            let destination_y = (y - crop_top) + pad_top;
            if destination_x < square.width() && destination_y < square.height() {
                square.put_pixel(destination_x, destination_y, *rotated.get_pixel(x, y));
            }
        }
    }
    let resized = image::imageops::resize(
        &square,
        HAND_INPUT_SIZE,
        HAND_INPUT_SIZE,
        FilterType::Triangle,
    );
    let input = resized
        .pixels()
        .flat_map(|pixel| pixel.0.map(|channel| channel as f32 / 255.0))
        .collect();
    let transform = HandPoseTransform {
        crop_bounds: [
            crop_left as f32,
            crop_top as f32,
            crop_right as f32,
            crop_bottom as f32,
        ],
        rotation_radians,
        canvas_center,
        source_offset: [offset_x as f32, offset_y as f32],
        source_width: width,
        source_height: height,
    };
    (input, transform)
}

fn decode_hand_landmarks(
    values: &[f32],
    confidence: &[f32],
    transform: HandPoseTransform,
) -> Result<Option<[[f32; 2]; HAND_LANDMARK_COUNT]>, String> {
    if values.len() != HAND_LANDMARK_COUNT * 3 || confidence.len() != 1 {
        return Err("Hand landmark model returned an unexpected output shape.".to_owned());
    }
    if confidence[0] < 0.8 {
        return Ok(None);
    }
    let scale = (transform.crop_bounds[2] - transform.crop_bounds[0])
        .max(transform.crop_bounds[3] - transform.crop_bounds[1])
        / HAND_INPUT_SIZE as f32;
    let crop_center = [
        (transform.crop_bounds[0] + transform.crop_bounds[2]) / 2.0,
        (transform.crop_bounds[1] + transform.crop_bounds[3]) / 2.0,
    ];
    let mut landmarks = [[0.0; 2]; HAND_LANDMARK_COUNT];
    for (index, landmark) in landmarks.iter_mut().enumerate() {
        let offset = index * 3;
        let upright_delta = [
            (values[offset] - HAND_INPUT_SIZE as f32 / 2.0) * scale,
            (values[offset + 1] - HAND_INPUT_SIZE as f32 / 2.0) * scale,
        ];
        let rotated_delta = rotate_point(upright_delta, [0.0, 0.0], transform.rotation_radians);
        let original_canvas_point = rotate_point(
            [
                crop_center[0] + rotated_delta[0],
                crop_center[1] + rotated_delta[1],
            ],
            transform.canvas_center,
            -transform.rotation_radians,
        );
        landmark[0] = (original_canvas_point[0] - transform.source_offset[0])
            .clamp(0.0, transform.source_width as f32);
        landmark[1] = (original_canvas_point[1] - transform.source_offset[1])
            .clamp(0.0, transform.source_height as f32);
    }
    Ok(Some(landmarks))
}

fn normalize_radians(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn rotate_point(point: [f32; 2], center: [f32; 2], radians: f32) -> [f32; 2] {
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    let (sin, cos) = radians.sin_cos();
    [
        cos * x + sin * y + center[0],
        -sin * x + cos * y + center[1],
    ]
}

fn rotate_image(image: &RgbImage, radians: f32, center: [f32; 2]) -> RgbImage {
    let mut rotated = RgbImage::new(image.width(), image.height());
    for y in 0..image.height() {
        for x in 0..image.width() {
            let source = rotate_point([x as f32, y as f32], center, -radians);
            let source_x = source[0].round() as i32;
            let source_y = source[1].round() as i32;
            if source_x >= 0
                && source_y >= 0
                && source_x < image.width() as i32
                && source_y < image.height() as i32
            {
                rotated.put_pixel(x, y, *image.get_pixel(source_x as u32, source_y as u32));
            }
        }
    }
    rotated
}

fn points_bounds(points: &[[f32; 2]; 7]) -> [f32; 4] {
    let mut bounds = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for [x, y] in points {
        bounds[0] = bounds[0].min(*x);
        bounds[1] = bounds[1].min(*y);
        bounds[2] = bounds[2].max(*x);
        bounds[3] = bounds[3].max(*y);
    }
    bounds
}

fn enlarge_bounds(bounds: [f32; 4], factor: f32, image_size: u32) -> [f32; 4] {
    let center_x = (bounds[0] + bounds[2]) / 2.0;
    let center_y = (bounds[1] + bounds[3]) / 2.0;
    let half_width = (bounds[2] - bounds[0]).max(1.0) * factor / 2.0;
    let half_height = (bounds[3] - bounds[1]).max(1.0) * factor / 2.0;
    [
        (center_x - half_width).clamp(0.0, image_size as f32),
        (center_y - half_height).clamp(0.0, image_size as f32),
        (center_x + half_width).clamp(0.0, image_size as f32),
        (center_y + half_height).clamp(0.0, image_size as f32),
    ]
}

#[derive(Clone, Copy)]
struct LetterboxTransform {
    pad_left: f32,
    pad_top: f32,
    source_width: u32,
    source_height: u32,
}

fn preprocess_palm_input(image: &RgbImage) -> (Vec<f32>, LetterboxTransform) {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let scale = (PALM_INPUT_SIZE as f32 / width as f32).min(PALM_INPUT_SIZE as f32 / height as f32);
    let resized_width = ((width as f32 * scale).floor() as u32).clamp(1, PALM_INPUT_SIZE);
    let resized_height = ((height as f32 * scale).floor() as u32).clamp(1, PALM_INPUT_SIZE);
    let resized =
        image::imageops::resize(image, resized_width, resized_height, FilterType::Triangle);
    let pad_left = (PALM_INPUT_SIZE - resized_width) / 2;
    let pad_top = (PALM_INPUT_SIZE - resized_height) / 2;
    let mut input = vec![0.0; (PALM_INPUT_SIZE * PALM_INPUT_SIZE * 3) as usize];
    for (x, y, pixel) in resized.enumerate_pixels() {
        let offset = (((y + pad_top) * PALM_INPUT_SIZE + x + pad_left) * 3) as usize;
        input[offset] = pixel[0] as f32 / 255.0;
        input[offset + 1] = pixel[1] as f32 / 255.0;
        input[offset + 2] = pixel[2] as f32 / 255.0;
    }
    (
        input,
        LetterboxTransform {
            pad_left: pad_left as f32 / scale,
            pad_top: pad_top as f32 / scale,
            source_width: width,
            source_height: height,
        },
    )
}

fn decode_palm_outputs(
    box_values: &[f32],
    score_values: &[f32],
    transform: LetterboxTransform,
) -> Result<Vec<PalmDetection>, String> {
    if box_values.len() != PALM_ANCHOR_COUNT * PALM_VALUES_PER_ANCHOR
        || score_values.len() != PALM_ANCHOR_COUNT
    {
        return Err("Palm model returned an unexpected output shape.".to_owned());
    }

    let anchors = palm_anchors();
    let image_scale = transform.source_width.max(transform.source_height) as f32;
    let mut candidates = Vec::new();
    for (index, [anchor_x, anchor_y]) in anchors.iter().copied().enumerate() {
        let confidence = sigmoid(score_values[index]);
        if confidence < 0.65 {
            continue;
        }
        let values = &box_values[index * PALM_VALUES_PER_ANCHOR..][..PALM_VALUES_PER_ANCHOR];
        let center_x =
            (values[0] / PALM_INPUT_SIZE as f32 + anchor_x) * image_scale - transform.pad_left;
        let center_y =
            (values[1] / PALM_INPUT_SIZE as f32 + anchor_y) * image_scale - transform.pad_top;
        let box_width = values[2] / PALM_INPUT_SIZE as f32 * image_scale;
        let box_height = values[3] / PALM_INPUT_SIZE as f32 * image_scale;
        let bounds = [
            (center_x - box_width / 2.0).clamp(0.0, transform.source_width as f32),
            (center_y - box_height / 2.0).clamp(0.0, transform.source_height as f32),
            (center_x + box_width / 2.0).clamp(0.0, transform.source_width as f32),
            (center_y + box_height / 2.0).clamp(0.0, transform.source_height as f32),
        ];
        if bounds[2] <= bounds[0] || bounds[3] <= bounds[1] {
            continue;
        }
        let mut landmarks = [[0.0; 2]; 7];
        for (landmark_index, landmark) in landmarks.iter_mut().enumerate() {
            let offset = 4 + landmark_index * 2;
            landmark[0] = ((values[offset] / PALM_INPUT_SIZE as f32 + anchor_x) * image_scale
                - transform.pad_left)
                .clamp(0.0, transform.source_width as f32);
            landmark[1] = ((values[offset + 1] / PALM_INPUT_SIZE as f32 + anchor_y) * image_scale
                - transform.pad_top)
                .clamp(0.0, transform.source_height as f32);
        }
        candidates.push(PalmDetection {
            bounds,
            landmarks,
            finger_landmarks: None,
            confidence,
        });
    }

    candidates.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    let mut selected: Vec<PalmDetection> = Vec::with_capacity(2);
    for candidate in candidates {
        if selected
            .iter()
            .all(|kept| intersection_over_union(candidate.bounds, kept.bounds) < 0.3)
        {
            selected.push(candidate);
            if selected.len() == 2 {
                break;
            }
        }
    }
    Ok(selected)
}

fn palm_anchors() -> Vec<[f32; 2]> {
    let mut anchors = Vec::with_capacity(PALM_ANCHOR_COUNT);
    for stride in [8.0_f32, 16.0] {
        let cells = (PALM_INPUT_SIZE as f32 / stride) as u32;
        let anchors_per_cell = if stride == 8.0 { 2 } else { 6 };
        for row in 0..cells {
            for column in 0..cells {
                let anchor = [
                    (column as f32 + 0.5) * stride / PALM_INPUT_SIZE as f32,
                    (row as f32 + 0.5) * stride / PALM_INPUT_SIZE as f32,
                ];
                anchors.extend(std::iter::repeat_n(anchor, anchors_per_cell));
            }
        }
    }
    anchors
}

fn sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

fn intersection_over_union(left: [f32; 4], right: [f32; 4]) -> f32 {
    let x1 = left[0].max(right[0]);
    let y1 = left[1].max(right[1]);
    let x2 = left[2].min(right[2]);
    let y2 = left[3].min(right[3]);
    let intersection = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let area_left = (left[2] - left[0]).max(0.0) * (left[3] - left[1]).max(0.0);
    let area_right = (right[2] - right[0]).max(0.0) * (right[3] - right[1]).max(0.0);
    let union = area_left + area_right - intersection;
    if union > 0.0 {
        intersection / union
    } else {
        0.0
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
    use super::{
        HAND_INPUT_SIZE, HAND_LANDMARK_COUNT, HandPoseTransform, HandTrackingModels,
        LANDMARK_MODEL, PALM_ANCHOR_COUNT, PALM_INPUT_SIZE, PALM_MODEL, PALM_VALUES_PER_ANCHOR,
        PalmDetection, decode_hand_landmarks, decode_palm_outputs, palm_anchors,
        preprocess_hand_input, preprocess_palm_input,
    };
    use image::{Rgb, RgbImage};

    #[test]
    fn bundled_models_are_present_and_distinct() {
        assert!(PALM_MODEL.starts_with(b"\x08\x07"));
        assert!(LANDMARK_MODEL.starts_with(b"\x08\x07"));
        assert_ne!(PALM_MODEL, LANDMARK_MODEL);
    }

    #[test]
    fn bundled_onnx_models_load_into_runtime_sessions() {
        let models = HandTrackingModels::load().expect("bundled ONNX models should load");

        assert_eq!(models.palm_detection.inputs().len(), 1);
        assert_eq!(models.hand_landmarks.inputs().len(), 1);
        assert_eq!(models.palm_detection.outputs().len(), 2);
        assert_eq!(models.hand_landmarks.outputs().len(), 4);
    }

    #[test]
    fn palm_model_runs_on_a_blank_frame_without_inventing_a_detection() {
        let mut models = HandTrackingModels::load().expect("bundled ONNX models should load");
        let palms = models
            .detect_palms(&RgbImage::new(320, 240))
            .expect("blank frame inference should succeed");

        assert!(palms.is_empty());
    }

    #[test]
    fn landmark_model_runs_with_a_palm_crop_tensor() {
        let mut models = HandTrackingModels::load().expect("bundled ONNX models should load");
        let image = RgbImage::from_pixel(320, 240, Rgb([80, 120, 160]));
        let palm = PalmDetection {
            bounds: [110.0, 70.0, 220.0, 210.0],
            landmarks: [
                [160.0, 190.0],
                [130.0, 145.0],
                [160.0, 120.0],
                [185.0, 145.0],
                [200.0, 160.0],
                [145.0, 175.0],
                [150.0, 160.0],
            ],
            finger_landmarks: None,
            confidence: 0.99,
        };

        let landmarks = models
            .infer_finger_landmarks(&image, &palm)
            .expect("the landmark session should accept its prepared crop");

        assert!(landmarks.is_none() || landmarks.is_some_and(|points| points.len() == 21));
    }

    #[test]
    fn palm_anchor_grid_matches_the_bundled_detector_output_count() {
        let anchors = palm_anchors();

        assert_eq!(anchors.len(), PALM_ANCHOR_COUNT);
        assert_eq!(anchors[0], [0.020833334, 0.020833334]);
        assert_eq!(anchors[1152], [0.041666668, 0.041666668]);
    }

    #[test]
    fn palm_input_is_rgb_normalized_and_letterboxed() {
        let mut image = RgbImage::new(2, 1);
        image.put_pixel(0, 0, Rgb([255, 0, 0]));
        image.put_pixel(1, 0, Rgb([0, 0, 255]));

        let (input, _) = preprocess_palm_input(&image);
        let first_image_row = (48 * PALM_INPUT_SIZE * 3) as usize;

        assert_eq!(
            input.len(),
            (PALM_INPUT_SIZE * PALM_INPUT_SIZE * 3) as usize
        );
        assert_eq!(&input[..3], &[0.0, 0.0, 0.0]);
        assert_eq!(
            &input[first_image_row..first_image_row + 3],
            &[1.0, 0.0, 0.0]
        );
    }

    #[test]
    fn palm_decoder_applies_confidence_threshold_and_non_maximum_suppression() {
        let image = RgbImage::new(200, 100);
        let (_, transform) = preprocess_palm_input(&image);
        let mut boxes = vec![0.0; PALM_ANCHOR_COUNT * PALM_VALUES_PER_ANCHOR];
        let mut scores = vec![-20.0; PALM_ANCHOR_COUNT];
        for (index, score) in [(0, 4.0), (1, 3.0)] {
            let offset = index * PALM_VALUES_PER_ANCHOR;
            boxes[offset] = 92.0;
            boxes[offset + 1] = 92.0;
            boxes[offset + 2] = 60.0;
            boxes[offset + 3] = 60.0;
            scores[index] = score;
        }

        let detections =
            decode_palm_outputs(&boxes, &scores, transform).expect("valid model output");

        assert_eq!(detections.len(), 1);
        assert!(detections[0].confidence > 0.95);
        assert!((detections[0].bounds[0] - 68.75).abs() < 0.01);
        assert!((detections[0].bounds[1] - 18.75).abs() < 0.01);
    }

    #[test]
    fn hand_crop_is_rgb_normalized_and_has_the_model_input_dimensions() {
        let image = RgbImage::from_pixel(64, 48, Rgb([255, 0, 0]));
        let palm = PalmDetection {
            bounds: [12.0, 10.0, 48.0, 42.0],
            landmarks: [
                [30.0, 35.0],
                [18.0, 23.0],
                [30.0, 18.0],
                [38.0, 22.0],
                [44.0, 27.0],
                [22.0, 31.0],
                [25.0, 29.0],
            ],
            finger_landmarks: None,
            confidence: 0.99,
        };

        let (input, transform) = preprocess_hand_input(&image, &palm);

        assert_eq!(
            input.len(),
            (HAND_INPUT_SIZE * HAND_INPUT_SIZE * 3) as usize
        );
        assert!(input.iter().all(|value| (0.0..=1.0).contains(value)));
        assert!(transform.crop_bounds[2] > transform.crop_bounds[0]);
        assert!(transform.crop_bounds[3] > transform.crop_bounds[1]);
    }

    #[test]
    fn hand_landmark_decoder_requires_confidence_and_maps_21_points() {
        let transform = HandPoseTransform {
            crop_bounds: [20.0, 30.0, 220.0, 230.0],
            rotation_radians: 0.0,
            canvas_center: [200.0, 200.0],
            source_offset: [100.0, 100.0],
            source_width: 320,
            source_height: 240,
        };
        let values = vec![112.0; HAND_LANDMARK_COUNT * 3];
        let landmarks = decode_hand_landmarks(&values, &[0.9], transform)
            .expect("valid landmark output")
            .expect("high-confidence pose");

        assert_eq!(landmarks.len(), HAND_LANDMARK_COUNT);
        assert_eq!(landmarks[0], [20.0, 30.0]);
        assert!(
            decode_hand_landmarks(&values, &[0.5], transform)
                .expect("low-confidence output should be valid")
                .is_none()
        );
    }

    #[test]
    fn hand_landmark_decoder_reverses_crop_rotation_and_source_padding() {
        let transform = HandPoseTransform {
            crop_bounds: [20.0, 30.0, 220.0, 230.0],
            rotation_radians: std::f32::consts::FRAC_PI_2,
            canvas_center: [200.0, 200.0],
            source_offset: [100.0, 100.0],
            source_width: 320,
            source_height: 240,
        };
        let mut values = vec![112.0; HAND_LANDMARK_COUNT * 3];
        values[0] = 132.0;

        let landmarks = decode_hand_landmarks(&values, &[0.9], transform)
            .expect("valid rotated landmark output")
            .expect("high-confidence pose");

        assert!((landmarks[0][0] - 187.85715).abs() < 0.001);
        assert!((landmarks[0][1] - 20.0).abs() < 0.001);
    }

    #[test]
    fn hand_landmark_decoder_rejects_unexpected_model_shapes() {
        let transform = HandPoseTransform {
            crop_bounds: [0.0, 0.0, 224.0, 224.0],
            rotation_radians: 0.0,
            canvas_center: [112.0, 112.0],
            source_offset: [0.0, 0.0],
            source_width: 224,
            source_height: 224,
        };

        assert!(decode_hand_landmarks(&[0.0; 62], &[0.9], transform).is_err());
        assert!(decode_hand_landmarks(&[0.0; 63], &[0.9, 0.8], transform).is_err());
    }
}
