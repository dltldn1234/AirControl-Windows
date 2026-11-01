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
}

struct CameraWorker {
    stop: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
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
                capture_loop(app, runtime, index, worker_stop, worker_finished);
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
) {
    set_camera_state(&runtime, &app, CameraState::Starting, None);
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(PREVIEW_MAX_WIDTH, PREVIEW_MAX_HEIGHT),
            FrameFormat::MJPEG,
            30,
        )));

    let mut camera = match Camera::new(index, requested) {
        Ok(camera) => camera,
        Err(error) => {
            fail_worker(&runtime, &app, &finished, error.to_string());
            return;
        }
    };

    if let Err(error) = camera.open_stream() {
        fail_worker(&runtime, &app, &finished, error.to_string());
        return;
    }

    set_camera_state(&runtime, &app, CameraState::Running, None);
    let mut last_preview = Instant::now();

    while !stop.load(Ordering::Acquire) {
        let frame = match camera.frame() {
            Ok(frame) => frame,
            Err(error) => {
                fail_worker(&runtime, &app, &finished, error.to_string());
                return;
            }
        };

        if last_preview.elapsed() >= PREVIEW_INTERVAL {
            if let Err(error) = emit_preview(&app, &frame) {
                log::warn!("Camera preview frame could not be encoded: {error}");
            }
            last_preview = Instant::now();
        }
    }

    let _ = camera.stop_stream();
    set_camera_state(&runtime, &app, CameraState::Stopped, None);
    finished.store(true, Ordering::Release);
}

fn emit_preview(app: &AppHandle, frame: &nokhwa::Buffer) -> Result<(), String> {
    let image = frame
        .decode_image::<RgbFormat>()
        .map_err(|error| format!("Frame decoding failed: {error}"))?;
    let (width, height) = preview_dimensions(image.width(), image.height());
    let resized = image::imageops::resize(&image, width, height, FilterType::Triangle);
    let mut jpeg = Cursor::new(Vec::new());
    JpegEncoder::new_with_quality(&mut jpeg, 72)
        .encode_image(&resized)
        .map_err(|error| format!("JPEG encoding failed: {error}"))?;
    let payload = CameraFrameEvent {
        jpeg_base64: base64::engine::general_purpose::STANDARD.encode(jpeg.into_inner()),
        width,
        height,
    };
    app.emit(CAMERA_FRAME_EVENT, payload)
        .map_err(|error| format!("Preview frame could not be sent to the UI: {error}"))
}

fn fail_worker(
    runtime: &Arc<Mutex<RuntimeSnapshot>>,
    app: &AppHandle,
    finished: &AtomicBool,
    message: String,
) {
    set_camera_state(runtime, app, CameraState::Error, Some(message));
    finished.store(true, Ordering::Release);
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
    use super::{decode_camera_index, encode_camera_index, preview_dimensions};
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
}
