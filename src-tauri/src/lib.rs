use aircontrol_core::RuntimeSnapshot;
use aircontrol_core::UserPreferences;
use tauri::{AppHandle, State};
use tauri_plugin_log::{Target, TargetKind};

mod settings;

#[derive(Default)]
pub struct RuntimeState {
    snapshot: std::sync::Mutex<RuntimeSnapshot>,
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
        .setup(|_app| {
            log::info!("AirControl started with system controls disabled");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_runtime_status,
            load_preferences,
            save_preferences
        ])
        .run(tauri::generate_context!())
        .expect("failed to launch AirControl");
}
