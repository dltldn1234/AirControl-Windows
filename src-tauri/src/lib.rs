use aircontrol_core::RuntimeSnapshot;
use aircontrol_core::UserPreferences;
use camera::{CameraDevice, CameraService};
use tauri::{AppHandle, Emitter, State};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::{Target, TargetKind};

mod camera;
mod cursor;
mod hand_tracking;
mod keyboard;
mod settings;
mod window_manager;

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
    cursor: State<'_, cursor::CursorController>,
    window_control: State<'_, window_manager::WindowGestureController>,
    keyboard: State<'_, keyboard::KeyboardController>,
    device_id: String,
) -> Result<(), String> {
    let preferences = settings::load(&app)?;
    camera_service.start(
        app,
        runtime.snapshot.clone(),
        device_id,
        camera::GestureControl {
            cursor: cursor.inner().clone(),
            window: window_control.inner().clone(),
            keyboard: keyboard.inner().clone(),
            pinch_sensitivity: preferences.pinch_sensitivity,
        },
    )
}

#[tauri::command]
fn stop_camera(
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    camera_service: State<'_, CameraService>,
    cursor: State<'_, cursor::CursorController>,
    window_control: State<'_, window_manager::WindowGestureController>,
    keyboard: State<'_, keyboard::KeyboardController>,
) -> Result<(), String> {
    let camera_result = camera_service.stop(&runtime.snapshot, &app);
    let cursor_result = cursor.disable();
    let window_result = window_control.disable();
    let keyboard_result = keyboard.reset();
    if let Some(window) = app.get_webview_window("air-keyboard") {
        let _ = window.hide();
    }
    set_control_mode(&runtime, &app, aircontrol_core::ControlMode::Disabled);
    camera_result
        .and(cursor_result)
        .and(window_result)
        .and(keyboard_result)
}

#[tauri::command]
fn set_cursor_control(
    enabled: bool,
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    cursor: State<'_, cursor::CursorController>,
    window_control: State<'_, window_manager::WindowGestureController>,
) -> Result<(), String> {
    if enabled {
        let snapshot = runtime
            .snapshot
            .lock()
            .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?
            .clone();
        let readiness = aircontrol_core::ControlReadiness {
            camera_running: snapshot.camera == aircontrol_core::CameraState::Running,
            hand_pose_detected: snapshot.hand_tracking
                == aircontrol_core::HandTrackingState::Detected,
        };
        if !readiness.can_enable() {
            return Err(
                "Start the camera and wait for all hand landmarks before enabling cursor control."
                    .to_owned(),
            );
        }
        if snapshot.control_mode != aircontrol_core::ControlMode::Disabled {
            return Err("Stop the active control mode before enabling cursor control.".to_owned());
        }
        let preferences = settings::load(&app)?;
        window_control.disable()?;
        cursor.enable(
            preferences.camera_mirrored,
            preferences.cursor_sensitivity,
            preferences.cursor_smoothing,
            preferences.scroll_sensitivity,
            preferences.reverse_scroll_direction,
            preferences.right_click_enabled,
        )?;
        set_control_mode(&runtime, &app, aircontrol_core::ControlMode::Cursor);
    } else {
        cursor.disable()?;
        window_control.disable()?;
        set_control_mode(&runtime, &app, aircontrol_core::ControlMode::Disabled);
    }
    Ok(())
}

#[tauri::command]
fn set_window_control(
    enabled: bool,
    handle: Option<String>,
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    cursor: State<'_, cursor::CursorController>,
    window_control: State<'_, window_manager::WindowGestureController>,
) -> Result<(), String> {
    if enabled {
        let snapshot = runtime
            .snapshot
            .lock()
            .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?
            .clone();
        let readiness = aircontrol_core::ControlReadiness {
            camera_running: snapshot.camera == aircontrol_core::CameraState::Running,
            hand_pose_detected: snapshot.hand_tracking
                == aircontrol_core::HandTrackingState::Detected,
        };
        if !readiness.can_enable() {
            return Err(
                "Start the camera and wait for all hand landmarks before enabling window control."
                    .to_owned(),
            );
        }
        if snapshot.control_mode != aircontrol_core::ControlMode::Disabled {
            return Err("Stop the active control mode before enabling window control.".to_owned());
        }
        let selected = handle.as_deref().ok_or_else(|| {
            "Choose an application window before enabling hand movement.".to_owned()
        })?;
        let preferences = settings::load(&app)?;
        cursor.disable()?;
        window_control.enable(
            selected,
            preferences.window_sensitivity,
            preferences.camera_mirrored,
            preferences.app_switch_enabled,
        )?;
        set_control_mode(&runtime, &app, aircontrol_core::ControlMode::Window);
    } else {
        window_control.disable()?;
        let current_mode = runtime
            .snapshot
            .lock()
            .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?
            .control_mode;
        if current_mode == aircontrol_core::ControlMode::Window {
            set_control_mode(&runtime, &app, aircontrol_core::ControlMode::Disabled);
        }
    }
    Ok(())
}

#[tauri::command]
fn set_air_keyboard_control(
    enabled: bool,
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    cursor: State<'_, cursor::CursorController>,
    window_control: State<'_, window_manager::WindowGestureController>,
    keyboard: State<'_, keyboard::KeyboardController>,
) -> Result<(), String> {
    if enabled {
        let snapshot = runtime
            .snapshot
            .lock()
            .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?
            .clone();
        let readiness = aircontrol_core::ControlReadiness {
            camera_running: snapshot.camera == aircontrol_core::CameraState::Running,
            hand_pose_detected: snapshot.hand_tracking
                == aircontrol_core::HandTrackingState::Detected,
        };
        if !readiness.can_enable() {
            return Err(
                "Start the camera and wait for hand tracking before enabling Air Keyboard."
                    .to_owned(),
            );
        }
        if snapshot.control_mode != aircontrol_core::ControlMode::Disabled {
            return Err("Stop the active control mode before enabling Air Keyboard.".to_owned());
        }
        let preferences = settings::load(&app)?;
        cursor.disable()?;
        window_control.disable()?;
        keyboard.reset()?;
        show_air_keyboard(&app)?;
        let Some(bounds) = keyboard::overlay_bounds(&app) else {
            if let Some(window) = app.get_webview_window("air-keyboard") {
                let _ = window.hide();
            }
            return Err("Windows did not report the floating keyboard position.".to_owned());
        };
        if let Err(error) = cursor.enable_air_keyboard(
            preferences.camera_mirrored,
            preferences.cursor_sensitivity,
            preferences.cursor_smoothing,
            bounds,
        ) {
            if let Some(window) = app.get_webview_window("air-keyboard") {
                let _ = window.hide();
            }
            return Err(error);
        }
        set_control_mode(&runtime, &app, aircontrol_core::ControlMode::AirKeyboard);
    } else {
        let cursor_result = cursor.disable();
        let keyboard_result = keyboard.reset();
        if let Some(window) = app.get_webview_window("air-keyboard") {
            let _ = window.hide();
        }
        let current_mode = runtime
            .snapshot
            .lock()
            .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?
            .control_mode;
        if current_mode == aircontrol_core::ControlMode::AirKeyboard {
            set_control_mode(&runtime, &app, aircontrol_core::ControlMode::Disabled);
        }
        cursor_result.and(keyboard_result)?;
    }
    Ok(())
}

#[tauri::command]
fn press_air_keyboard_key(
    key: String,
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    keyboard: State<'_, keyboard::KeyboardController>,
) -> Result<(), String> {
    let snapshot = runtime
        .snapshot
        .lock()
        .map_err(|_| "Runtime status is temporarily unavailable.".to_owned())?;
    if snapshot.control_mode != aircontrol_core::ControlMode::AirKeyboard {
        return Err("Enable Air Keyboard before sending a key.".to_owned());
    }
    let window = app
        .get_webview_window("air-keyboard")
        .ok_or_else(|| "The Air Keyboard surface is not available.".to_owned())?;
    if !window.is_visible().unwrap_or(false) {
        return Err("Show Air Keyboard before sending a key.".to_owned());
    }
    keyboard.press(&key)
}

fn show_air_keyboard(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("air-keyboard") {
        return window
            .show()
            .map_err(|error| format!("Air Keyboard could not be shown: {error}"));
    }
    WebviewWindowBuilder::new(
        app,
        "air-keyboard",
        WebviewUrl::App("index.html?surface=keyboard".into()),
    )
    .title("Air Keyboard")
    .inner_size(760.0, 304.0)
    .min_inner_size(560.0, 270.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .focusable(false)
    .focused(false)
    .center()
    .build()
    .map(|_| ())
    .map_err(|error| format!("Air Keyboard window could not be created: {error}"))
}

#[tauri::command]
fn list_control_windows() -> Result<Vec<window_manager::WindowInfo>, String> {
    window_manager::list_windows()
}

#[tauri::command]
fn get_foreground_control_window() -> Result<Option<window_manager::WindowInfo>, String> {
    window_manager::foreground_window()
}

#[tauri::command]
fn focus_control_window(handle: String) -> Result<(), String> {
    window_manager::focus_window(&handle)
}

#[tauri::command]
fn move_control_window(
    handle: String,
    delta: [f32; 2],
) -> Result<aircontrol_core::window::WindowRect, String> {
    window_manager::move_window_by(&handle, delta)
}

#[tauri::command]
fn resize_control_window(
    handle: String,
    scale: f32,
) -> Result<aircontrol_core::window::WindowRect, String> {
    window_manager::resize_window_by(&handle, scale)
}

#[tauri::command]
fn minimize_control_window(handle: String) -> Result<(), String> {
    window_manager::minimize_window(&handle)
}

#[tauri::command]
fn maximize_control_window(handle: String) -> Result<(), String> {
    window_manager::maximize_window(&handle)
}

fn set_control_mode(runtime: &RuntimeState, app: &AppHandle, mode: aircontrol_core::ControlMode) {
    if let Ok(mut snapshot) = runtime.snapshot.lock() {
        snapshot.control_mode = mode;
        snapshot.active_gesture = match mode {
            aircontrol_core::ControlMode::Cursor => aircontrol_core::ActiveGesture::Cursor,
            aircontrol_core::ControlMode::Disabled
            | aircontrol_core::ControlMode::Window
            | aircontrol_core::ControlMode::AirKeyboard => aircontrol_core::ActiveGesture::None,
        };
        let _ = app.emit("runtime-status", snapshot.clone());
    }
}

fn set_window_gesture(
    runtime: &std::sync::Arc<std::sync::Mutex<RuntimeSnapshot>>,
    app: &AppHandle,
    gesture: aircontrol_core::ActiveGesture,
) {
    if let Ok(mut snapshot) = runtime.lock()
        && snapshot.control_mode == aircontrol_core::ControlMode::Window
    {
        snapshot.active_gesture = gesture;
        let _ = app.emit("runtime-status", snapshot.clone());
    }
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
        .manage(cursor::CursorController::default())
        .manage(window_manager::WindowGestureController::default())
        .manage(keyboard::KeyboardController::default())
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
            stop_camera,
            set_cursor_control,
            set_window_control,
            set_air_keyboard_control,
            press_air_keyboard_key,
            list_control_windows,
            get_foreground_control_window,
            focus_control_window,
            move_control_window,
            resize_control_window,
            minimize_control_window,
            maximize_control_window
        ])
        .run(tauri::generate_context!())
        .expect("failed to launch AirControl");
}
