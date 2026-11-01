use crate::cursor::CursorController;
use crate::hand_tracking::HandTrackingModels;
use crate::keyboard::KeyboardController;
use crate::window_manager::WindowGestureController;
use aircontrol_core::command_palette::CommandPaletteStateMachine;
use aircontrol_core::gesture::{PinchAction, next_cursor_gesture};
use aircontrol_core::hand_tracking::HandTrackAssociator;
use aircontrol_core::{CameraState, RuntimeSnapshot};
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
    Resolution,
};
use nokhwa::{Camera, query};
use serde::Serialize;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const CAMERA_STATUS_EVENT: &str = "camera-status";
const CAMERA_FRAME_EVENT: &str = "camera-frame";
const PREVIEW_MAX_WIDTH: u32 = 640;
const PREVIEW_MAX_HEIGHT: u32 = 480;
const PREVIEW_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraDevice {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CameraStatusEvent {
    state: CameraState,
    message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CameraFrameEvent {
    jpeg_base64: String,
    width: u32,
    height: u32,
    palms: Vec<crate::hand_tracking::PalmDetection>,
}

struct CameraWorker {
    stop: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

pub(super) struct GestureControl {
    pub(super) cursor: CursorController,
    pub(super) window: WindowGestureController,
    pub(super) keyboard: KeyboardController,
    pub(super) pinch_sensitivity: f32,
    pub(super) camera_mirrored: bool,
}

#[derive(Default)]
pub struct CameraService {
    worker: Mutex<Option<CameraWorker>>,
}

impl CameraService {
    pub fn list_devices(&self) -> Result<Vec<CameraDevice>, String> {
        request_platform_camera_access()?;
        query(native_api_backend())
            .map(|cameras| {
                cameras
                    .into_iter()
                    .map(|camera| CameraDevice {
                        id: encode_camera_index(camera.index()),
                        name: camera.human_name(),
                    })
                    .collect()
            })
            .map_err(|error| format!("Camera devices could not be listed: {error}"))
    }

    pub fn start(
        &self,
        app: AppHandle,
        runtime: Arc<Mutex<RuntimeSnapshot>>,
        device_id: String,
        controls: GestureControl,
    ) -> Result<(), String> {
        let index = decode_camera_index(&device_id)?;
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Camera service is temporarily unavailable.".to_owned())?;

        if let Some(existing) = worker.as_ref()
            && !existing.finished.load(Ordering::Acquire)
        {
            return Err("Camera capture is already running.".to_owned());
        }
        if let Some(mut finished_worker) = worker.take()
            && let Some(thread) = finished_worker.thread.take()
        {
            let _ = thread.join();
        }

        let stop = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_finished = Arc::clone(&finished);
        let thread = thread::Builder::new()
            .name("aircontrol-camera".to_owned())
            .spawn(move || {
                capture_loop(app, runtime, index, worker_stop, worker_finished, controls);
            })
            .map_err(|error| format!("Camera worker could not be started: {error}"))?;

        *worker = Some(CameraWorker {
            stop,
            finished,
            thread: Some(thread),
        });
        Ok(())
    }

    pub fn stop(
        &self,
        runtime: &Arc<Mutex<RuntimeSnapshot>>,
        app: &AppHandle,
    ) -> Result<(), String> {
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Camera service is temporarily unavailable.".to_owned())?;
        if let Some(mut worker) = worker.take() {
            worker.stop.store(true, Ordering::Release);
            if let Some(thread) = worker.thread.take() {
                thread
                    .join()
                    .map_err(|_| "Camera worker stopped unexpectedly.".to_owned())?;
            }
        }
        set_camera_state(runtime, app, CameraState::Stopped, None);
        set_tracking_state(
            runtime,
            app,
            aircontrol_core::HandTrackingState::Waiting,
            None,
        );
        Ok(())
    }
}

impl Drop for CameraService {
    fn drop(&mut self) {
        if let Ok(worker) = self.worker.get_mut()
            && let Some(mut worker) = worker.take()
        {
            worker.stop.store(true, Ordering::Release);
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

fn capture_loop(
    app: AppHandle,
    runtime: Arc<Mutex<RuntimeSnapshot>>,
    index: CameraIndex,
    stop: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    controls: GestureControl,
) {
    let GestureControl {
        cursor,
        window: window_control,
        keyboard,
        pinch_sensitivity,
        camera_mirrored,
    } = controls;
    set_camera_state(&runtime, &app, CameraState::Starting, None);
    set_tracking_state(
        &runtime,
        &app,
        aircontrol_core::HandTrackingState::ModelLoading,
        None,
    );
    let mut hand_models = match HandTrackingModels::load() {
        Ok(models) => {
            set_tracking_state(
                &runtime,
                &app,
                aircontrol_core::HandTrackingState::ModelReady,
                Some("ONNX models loaded; waiting for camera frames."),
            );
            Some(models)
        }
        Err(error) => {
            set_tracking_state(
                &runtime,
                &app,
                aircontrol_core::HandTrackingState::Error,
                Some(&error),
            );
            log::error!("Hand tracking models could not be loaded: {error}");
            None
        }
    };
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(PREVIEW_MAX_WIDTH, PREVIEW_MAX_HEIGHT),
            FrameFormat::MJPEG,
            30,
        )));

    let mut camera = match Camera::new(index, requested) {
        Ok(camera) => camera,
        Err(error) => {
            fail_worker(
                &runtime,
                &app,
                &finished,
                &cursor,
                &window_control,
                &keyboard,
                error.to_string(),
            );
            return;
        }
    };

    if let Err(error) = camera.open_stream() {
        fail_worker(
            &runtime,
            &app,
            &finished,
            &cursor,
            &window_control,
            &keyboard,
            error.to_string(),
        );
        return;
    }

    set_camera_state(&runtime, &app, CameraState::Running, None);
    let mut last_preview = Instant::now();
    let mut hand_tracks = HandTrackAssociator::default();
    let mut command_palette = CommandPaletteStateMachine::default();

    while !stop.load(Ordering::Acquire) {
        let frame = match camera.frame() {
            Ok(frame) => frame,
            Err(error) => {
                fail_worker(
                    &runtime,
                    &app,
                    &finished,
                    &cursor,
                    &window_control,
                    &keyboard,
                    error.to_string(),
                );
                return;
            }
        };

        if last_preview.elapsed() >= PREVIEW_INTERVAL {
            match frame.decode_image::<RgbFormat>() {
                Ok(image) => {
                    let inference = hand_models
                        .as_mut()
                        .map(|models| models.detect_palms(&image));
                    let palms = match inference {
                        Some(Ok(palms)) => {
                            let keyboard_active = runtime.lock().is_ok_and(|snapshot| {
                                snapshot.control_mode == aircontrol_core::ControlMode::AirKeyboard
                            });
                            let keyboard_bounds = keyboard_active
                                .then(|| crate::keyboard::overlay_bounds(&app))
                                .flatten();
                            let _ = cursor.update_keyboard_bounds(keyboard_bounds);
                            let has_pose = palms.iter().any(|palm| palm.finger_landmarks.is_some());
                            if !has_pose {
                                let _ = keyboard.reset_modifiers();
                            }
                            let state = if has_pose {
                                aircontrol_core::HandTrackingState::Detected
                            } else if palms.is_empty() {
                                aircontrol_core::HandTrackingState::Lost
                            } else {
                                aircontrol_core::HandTrackingState::PalmDetected
                            };
                            set_tracking_state(&runtime, &app, state, None);
                            let tracked_hands = associate_hand_landmarks(
                                &mut hand_tracks,
                                &palms,
                                [image.width(), image.height()],
                            );
                            update_command_palette(
                                &mut command_palette,
                                &runtime,
                                &app,
                                tracked_hands.iter().flatten().next().copied(),
                                [image.width(), image.height()],
                                camera_mirrored,
                            );
                            let landmarks = tracked_hands.iter().flatten().next().copied();
                            match cursor.update_hand_landmarks(
                                landmarks,
                                [image.width(), image.height()],
                                pinch_sensitivity,
                            ) {
                                Ok(action) => set_cursor_gesture(&runtime, &app, action),
                                Err(error) => {
                                    log::warn!(
                                        "Cursor control stopped after an input error: {error}"
                                    );
                                    let _ = cursor.disable();
                                    let _ = window_control.disable();
                                    let _ = keyboard.reset();
                                    set_control_disabled(&runtime, &app);
                                }
                            }
                            match window_control.update_hand_landmarks(
                                &tracked_hands,
                                [image.width(), image.height()],
                                pinch_sensitivity,
                            ) {
                                Ok(gesture) => crate::set_window_gesture(&runtime, &app, gesture),
                                Err(error) => {
                                    log::warn!(
                                        "Window control stopped after an input error: {error}"
                                    );
                                    let _ = window_control.disable();
                                    let _ = cursor.disable();
                                    let _ = keyboard.reset();
                                    set_control_disabled(&runtime, &app);
                                }
                            }
                            palms
                        }
                        Some(Err(error)) => {
                            hand_models = None;
                            hand_tracks.reset();
                            let _ = cursor.update_hand_landmarks(
                                None,
                                [image.width(), image.height()],
                                pinch_sensitivity,
                            );
                            let _ = cursor.disable();
                            let _ = window_control.disable();
                            let _ = keyboard.reset();
                            set_control_disabled(&runtime, &app);
                            set_tracking_state(
                                &runtime,
                                &app,
                                aircontrol_core::HandTrackingState::Error,
                                Some(&error),
                            );
                            log::error!("Hand inference stopped after a model error: {error}");
                            Vec::new()
                        }
                        None => {
                            hand_tracks.reset();
                            let _ = keyboard.reset_modifiers();
                            let _ = cursor.update_hand_landmarks(
                                None,
                                [image.width(), image.height()],
                                pinch_sensitivity,
                            );
                            let _ = window_control.update_hand_landmarks(
                                &[],
                                [image.width(), image.height()],
                                pinch_sensitivity,
                            );
                            crate::set_window_gesture(
                                &runtime,
                                &app,
                                aircontrol_core::ActiveGesture::None,
                            );
                            Vec::new()
                        }
                    };
                    if let Err(error) = emit_preview(&app, &image, &palms) {
                        log::warn!("Camera preview frame could not be encoded: {error}");
                    }
                }
                Err(error) => log::warn!("Camera preview frame could not be decoded: {error}"),
            }
            last_preview = Instant::now();
        }
    }

    let _ = camera.stop_stream();
    let _ = cursor.disable();
    let _ = window_control.disable();
    let _ = keyboard.reset();
    set_control_disabled(&runtime, &app);
    set_camera_state(&runtime, &app, CameraState::Stopped, None);
    finished.store(true, Ordering::Release);
}

fn update_command_palette(
    machine: &mut CommandPaletteStateMachine,
    runtime: &Arc<Mutex<RuntimeSnapshot>>,
    app: &AppHandle,
    landmarks: Option<[[f32; 2]; 21]>,
    frame_size: [u32; 2],
    mirrored: bool,
) {
    let palette_mode = runtime.lock().is_ok_and(|snapshot| {
        snapshot.control_mode == aircontrol_core::ControlMode::CommandPalette
    });
    if !palette_mode {
        if machine.is_active() {
            let _ = machine.mode_changed();
        }
        return;
    }
    if !machine.is_active() {
        machine.open();
    }
    let now = Instant::now();
    let update = match landmarks.and_then(|points| palette_sample(points, frame_size, mirrored)) {
        Some((position, pinch_distance, cancel_pose)) => {
            machine.update(Some(position), Some(pinch_distance), cancel_pose, now)
        }
        None => machine.tracking_lost(),
    };
    let mut message = None;
    if let Some(command) = update.command {
        message = Some(
            match crate::command_palette::execute_if_active(runtime, command) {
                Ok(()) => format!("Ran {}", command.id()),
                Err(error) => error,
            },
        );
    } else if update.cancelled {
        message = Some("Command palette cancelled; no command was sent.".to_owned());
    }
    if let Ok(mut snapshot) = runtime.lock() {
        if snapshot.control_mode != aircontrol_core::ControlMode::CommandPalette {
            return;
        }
        snapshot.palette_hovered_index = update.hovered_index;
        if let Some(message) = message {
            snapshot.palette_message = Some(message);
        }
        if update.cancelled {
            snapshot.control_mode = aircontrol_core::ControlMode::Disabled;
            snapshot.active_gesture = aircontrol_core::ActiveGesture::None;
            snapshot.palette_hovered_index = None;
        } else if update.hovered_index.is_some() {
            snapshot.active_gesture = aircontrol_core::ActiveGesture::CommandPalette;
        }
        let _ = app.emit("runtime-status", snapshot.clone());
    }
}

fn palette_sample(
    landmarks: [[f32; 2]; 21],
    frame_size: [u32; 2],
    mirrored: bool,
) -> Option<([f32; 2], f32, bool)> {
    let [width, height] = frame_size;
    if width == 0 || height == 0 || landmarks.iter().flatten().any(|value| !value.is_finite()) {
        return None;
    }
    let mut position = [
        landmarks[8][0] / width as f32,
        landmarks[8][1] / height as f32,
    ];
    if mirrored {
        position[0] = 1.0 - position[0];
    }
    let palm_scale = distance(landmarks[0], landmarks[9]);
    if !palm_scale.is_finite() || palm_scale <= f32::EPSILON {
        return None;
    }
    let pinch_distance = distance(landmarks[4], landmarks[8]) / palm_scale;
    let cancel_pose = [8, 12, 16, 20]
        .into_iter()
        .zip([5, 9, 13, 17])
        .all(|(tip, base)| {
            distance(landmarks[tip], landmarks[0]) < distance(landmarks[base], landmarks[0]) * 0.92
        });
    Some((position, pinch_distance, cancel_pose))
}

fn distance(first: [f32; 2], second: [f32; 2]) -> f32 {
    (first[0] - second[0]).hypot(first[1] - second[1])
}

fn emit_preview(
    app: &AppHandle,
    image: &image::RgbImage,
    palms: &[crate::hand_tracking::PalmDetection],
) -> Result<(), String> {
    let (width, height) = preview_dimensions(image.width(), image.height());
    let resized = image::imageops::resize(image, width, height, FilterType::Triangle);
    let mut jpeg = Cursor::new(Vec::new());
    JpegEncoder::new_with_quality(&mut jpeg, 72)
        .encode_image(&resized)
        .map_err(|error| format!("JPEG encoding failed: {error}"))?;
    let payload = CameraFrameEvent {
        jpeg_base64: base64::engine::general_purpose::STANDARD.encode(jpeg.into_inner()),
        width,
        height,
        palms: palms.to_vec(),
    };
    app.emit(CAMERA_FRAME_EVENT, payload)
        .map_err(|error| format!("Preview frame could not be sent to the UI: {error}"))
}

fn associate_hand_landmarks(
    associator: &mut HandTrackAssociator,
    palms: &[crate::hand_tracking::PalmDetection],
    frame_size: [u32; 2],
) -> [Option<[[f32; 2]; 21]>; 2] {
    let [width, height] = frame_size;
    if width == 0 || height == 0 {
        associator.reset();
        return [None; 2];
    }
    let centers = palms
        .iter()
        .map(|palm| {
            palm.finger_landmarks.map(|landmarks| {
                [
                    landmarks[0][0] / width as f32,
                    landmarks[0][1] / height as f32,
                ]
            })
        })
        .collect::<Vec<_>>();
    associator
        .update(&centers)
        .map(|index| index.and_then(|index| palms.get(index)?.finger_landmarks))
}

#[cfg(test)]
mod hand_association_tests {
    use super::*;
    use crate::hand_tracking::PalmDetection;

    fn palm_at(x: f32, confidence: f32) -> PalmDetection {
        let mut finger_landmarks = [[0.0; 2]; 21];
        finger_landmarks[0] = [x, 50.0];
        PalmDetection {
            bounds: [0.0; 4],
            landmarks: [[0.0; 2]; 7],
            finger_landmarks: Some(finger_landmarks),
            confidence,
        }
    }

    #[test]
    fn camera_hand_landmarks_follow_tracks_not_detection_confidence_order() {
        let mut associator = HandTrackAssociator::default();
        let first = [palm_at(20.0, 0.9), palm_at(80.0, 0.8)];
        let tracks = associate_hand_landmarks(&mut associator, &first, [100, 100]);
        assert_eq!(tracks[0].unwrap()[0], [20.0, 50.0]);
        assert_eq!(tracks[1].unwrap()[0], [80.0, 50.0]);

        let reordered = [palm_at(81.0, 0.95), palm_at(19.0, 0.7)];
        let tracks = associate_hand_landmarks(&mut associator, &reordered, [100, 100]);
        assert_eq!(tracks[0].unwrap()[0], [19.0, 50.0]);
        assert_eq!(tracks[1].unwrap()[0], [81.0, 50.0]);
    }

    #[test]
    fn camera_uses_the_remaining_hand_without_reusing_lost_landmarks() {
        let mut associator = HandTrackAssociator::default();
        let both = [palm_at(20.0, 0.9), palm_at(80.0, 0.8)];
        associate_hand_landmarks(&mut associator, &both, [100, 100]);

        let remaining = [palm_at(82.0, 0.95)];
        let tracks = associate_hand_landmarks(&mut associator, &remaining, [100, 100]);

        assert!(tracks[0].is_none());
        assert_eq!(tracks[1].unwrap()[0], [82.0, 50.0]);
        assert_eq!(
            tracks.iter().flatten().next().copied().unwrap()[0],
            [82.0, 50.0]
        );
    }

    #[test]
    fn camera_association_resets_on_invalid_frame_geometry() {
        let mut associator = HandTrackAssociator::default();
        let palms = [palm_at(20.0, 0.9)];
        associate_hand_landmarks(&mut associator, &palms, [100, 100]);

        assert_eq!(
            associate_hand_landmarks(&mut associator, &palms, [0, 100]),
            [None, None]
        );
        let far = [palm_at(90.0, 0.9)];
        let tracks = associate_hand_landmarks(&mut associator, &far, [100, 100]);
        assert_eq!(tracks[0].unwrap()[0], [90.0, 50.0]);
    }
}

fn fail_worker(
    runtime: &Arc<Mutex<RuntimeSnapshot>>,
    app: &AppHandle,
    finished: &AtomicBool,
    cursor: &CursorController,
    window_control: &WindowGestureController,
    keyboard: &KeyboardController,
    message: String,
) {
    let _ = cursor.disable();
    let _ = window_control.disable();
    let _ = keyboard.reset();
    set_control_disabled(runtime, app);
    set_camera_state(runtime, app, CameraState::Error, Some(message));
    finished.store(true, Ordering::Release);
}

fn set_control_disabled(runtime: &Arc<Mutex<RuntimeSnapshot>>, app: &AppHandle) {
    if let Ok(mut snapshot) = runtime.lock() {
        snapshot.control_mode = aircontrol_core::ControlMode::Disabled;
        snapshot.active_gesture = aircontrol_core::ActiveGesture::None;
        snapshot.palette_hovered_index = None;
        snapshot.palette_message = None;
        let _ = app.emit("runtime-status", snapshot.clone());
    }
}

fn set_cursor_gesture(runtime: &Arc<Mutex<RuntimeSnapshot>>, app: &AppHandle, action: PinchAction) {
    if let Ok(mut snapshot) = runtime.lock()
        && snapshot.control_mode == aircontrol_core::ControlMode::Cursor
    {
        let next = next_cursor_gesture(snapshot.active_gesture, action);
        if next != snapshot.active_gesture {
            snapshot.active_gesture = next;
            let _ = app.emit("runtime-status", snapshot.clone());
        }
    }
}

fn set_camera_state(
    runtime: &Arc<Mutex<RuntimeSnapshot>>,
    app: &AppHandle,
    state: CameraState,
    message: Option<String>,
) {
    if let Ok(mut snapshot) = runtime.lock() {
        snapshot.camera = state;
    }
    let _ = app.emit(CAMERA_STATUS_EVENT, CameraStatusEvent { state, message });
}

fn set_tracking_state(
    runtime: &Arc<Mutex<RuntimeSnapshot>>,
    app: &AppHandle,
    state: aircontrol_core::HandTrackingState,
    message: Option<&str>,
) {
    if let Ok(mut snapshot) = runtime.lock() {
        snapshot.hand_tracking = state;
    }
    let _ = app.emit(
        "hand-tracking-status",
        serde_json::json!({ "state": state, "message": message }),
    );
}

fn encode_camera_index(index: &CameraIndex) -> String {
    if index.is_index() {
        format!("index:{}", index.as_index().unwrap_or_default())
    } else {
        format!("string:{}", index.as_string())
    }
}

fn decode_camera_index(id: &str) -> Result<CameraIndex, String> {
    if let Some(value) = id.strip_prefix("index:") {
        return value
            .parse::<u32>()
            .map(CameraIndex::Index)
            .map_err(|_| "The selected camera identifier is invalid.".to_owned());
    }
    if let Some(value) = id.strip_prefix("string:") {
        if value.is_empty() {
            return Err("The selected camera identifier is invalid.".to_owned());
        }
        return Ok(CameraIndex::String(value.to_owned()));
    }
    Err("The selected camera identifier is invalid.".to_owned())
}

fn preview_dimensions(width: u32, height: u32) -> (u32, u32) {
    if width <= PREVIEW_MAX_WIDTH && height <= PREVIEW_MAX_HEIGHT {
        return (width.max(1), height.max(1));
    }

    let width_scale = PREVIEW_MAX_WIDTH as f32 / width.max(1) as f32;
    let height_scale = PREVIEW_MAX_HEIGHT as f32 / height.max(1) as f32;
    let scale = width_scale.min(height_scale);
    (
        ((width as f32 * scale).round() as u32).max(1),
        ((height as f32 * scale).round() as u32).max(1),
    )
}

fn native_api_backend() -> ApiBackend {
    nokhwa::native_api_backend().unwrap_or(ApiBackend::Auto)
}

#[cfg(target_os = "macos")]
fn request_platform_camera_access() -> Result<(), String> {
    if nokhwa::nokhwa_check() {
        return Ok(());
    }
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    nokhwa::nokhwa_initialize(move |granted| {
        let _ = sender.send(granted);
    });
    match receiver.recv_timeout(Duration::from_secs(60)) {
        Ok(true) => Ok(()),
        Ok(false) => Err("Camera permission was denied in macOS settings.".to_owned()),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            Err("Timed out while waiting for camera permission.".to_owned())
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("Camera permission request ended unexpectedly.".to_owned())
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn request_platform_camera_access() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{decode_camera_index, encode_camera_index, palette_sample, preview_dimensions};
    use nokhwa::utils::CameraIndex;

    #[test]
    fn camera_indices_preserve_numeric_and_string_forms() {
        for index in [
            CameraIndex::Index(2),
            CameraIndex::String("device-id".to_owned()),
        ] {
            let decoded = decode_camera_index(&encode_camera_index(&index)).expect("valid index");
            assert_eq!(decoded, index);
        }
    }

    #[test]
    fn invalid_camera_indices_are_rejected() {
        assert!(decode_camera_index("index:bad").is_err());
        assert!(decode_camera_index("string:").is_err());
        assert!(decode_camera_index("2").is_err());
    }

    #[test]
    fn preview_dimensions_preserve_aspect_ratio_within_bounds() {
        assert_eq!(preview_dimensions(1920, 1080), (640, 360));
        assert_eq!(preview_dimensions(320, 240), (320, 240));
        assert_eq!(preview_dimensions(0, 0), (1, 1));
    }

    #[test]
    fn palette_sample_mirrors_index_position_and_normalizes_pinch_by_palm_size() {
        let mut hand = [[0.0; 2]; 21];
        hand[0] = [50.0, 50.0];
        hand[4] = [22.0, 60.0];
        hand[8] = [20.0, 60.0];
        hand[9] = [50.0, 30.0];

        let (position, pinch, is_fist) =
            palette_sample(hand, [100, 100], true).expect("valid hand");

        assert_eq!(position, [0.8, 0.6]);
        assert!((pinch - 0.1).abs() < 0.001);
        assert!(!is_fist);
    }

    #[test]
    fn palette_sample_detects_closed_fist_cancel_pose() {
        let mut hand = [[0.0; 2]; 21];
        hand[0] = [50.0, 50.0];
        hand[9] = [40.0, 40.0];
        for (tip, base) in [(8, 5), (12, 9), (16, 13), (20, 17)] {
            hand[tip] = [51.0, 51.0];
            hand[base] = [40.0, 40.0];
        }
        hand[4] = [52.0, 52.0];

        let (_, _, is_fist) = palette_sample(hand, [100, 100], false).expect("valid hand");

        assert!(is_fist);
    }
}
