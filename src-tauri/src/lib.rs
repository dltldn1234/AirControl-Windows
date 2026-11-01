use aircontrol_core::RuntimeSnapshot;
use aircontrol_core::UserPreferences;
use camera::{CameraDevice, CameraService};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_log::{Target, TargetKind};

mod camera;
mod hand_tracking;
mod settings;

#[derive(Clone, Default)]
pub struct RuntimeState {
    snapshot: std::sync::Arc<std::sync::Mutex<RuntimeSnapshot>>,
}

#[tauri::command]
fn get_runtime_status(state: State<'_, RuntimeState>) -> Result<RuntimeSnapshot, String> {
    state
        .snapshot
        .lock()
        .map(|snapshot| snapshot.clone())
        .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())
}

#[tauri::command]
fn load_preferences(app: AppHandle) -> Result<UserPreferences, String> {
    settings::load(&app)
}

#[tauri::command]
fn save_preferences(
    app: AppHandle,
    preferences: UserPreferences,
) -> Result<UserPreferences, String> {
    settings::save(&app, preferences)
}

#[tauri::command]
fn list_cameras(
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    camera_service: State<'_, CameraService>,
) -> Result<Vec<CameraDevice>, String> {
    set_camera_state(
        &runtime,
        &app,
        aircontrol_core::CameraState::Enumerating,
        None,
    );
    match camera_service.list_devices() {
        Ok(devices) => {
            let state = if devices.is_empty() {
                aircontrol_core::CameraState::Unavailable
            } else {
                aircontrol_core::CameraState::Stopped
            };
            set_camera_state(&runtime, &app, state, None);
            Ok(devices)
        }
        Err(error) => {
            let state = if error.contains("permission") || error.contains("Permission") {
                aircontrol_core::CameraState::PermissionRequired
            } else {
                aircontrol_core::CameraState::Error
            };
            set_camera_state(&runtime, &app, state, Some(error.clone()));
            Err(error)
        }
    }
}

#[tauri::command]
fn start_camera(
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    camera_service: State<'_, CameraService>,
    device_id: String,
) -> Result<(), String> {
    camera_service.start(app, runtime.snapshot.clone(), device_id)
}

#[tauri::command]
fn stop_camera(
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    camera_service: State<'_, CameraService>,
) -> Result<(), String> {
    camera_service.stop(&runtime.snapshot, &app)
}

fn set_camera_state(
    runtime: &RuntimeState,
    app: &AppHandle,
    state: aircontrol_core::CameraState,
    message: Option<String>,
) {
    if let Ok(mut snapshot) = runtime.snapshot.lock() {
        snapshot.camera = state;
    }
    let _ = app.emit(
        "camera-status",
        serde_json::json!({ "state": state, "message": message }),
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(Target::new(TargetKind::LogDir {
                    file_name: Some("aircontrol".to_owned()),
                }))
                .build(),
        )
        .manage(RuntimeState::default())
        .manage(CameraService::default())
        .setup(|_app| {
            log::info!("AirControl started with system controls disabled");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_runtime_status,
            load_preferences,
            save_preferences,
            list_cameras,
            start_camera,
            stop_camera
        ])
        .run(tauri::generate_context!())
        .expect("failed to launch AirControl");
}
