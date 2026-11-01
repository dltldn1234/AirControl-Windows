use aircontrol_core::UserPreferences;
use std::fs;
use std::io::Write;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE: &str = "preferences.json";

pub fn load(app: &AppHandle) -> Result<UserPreferences, String> {
    load_from_path(&settings_path(app)?)
}

fn load_from_path(path: &std::path::Path) -> Result<UserPreferences, String> {
    match fs::read(path) {
        Ok(contents) => {
            let preferences = serde_json::from_slice::<UserPreferences>(&contents)
                .map_err(|error| format!("Saved preferences could not be read: {error}"))?;
            preferences.validate_profiles()?;
            Ok(preferences.normalized())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(UserPreferences::default())
        }
        Err(error) => Err(format!("Saved preferences could not be opened: {error}")),
    }
}

pub fn save(app: &AppHandle, preferences: UserPreferences) -> Result<UserPreferences, String> {
    let path = settings_path(app)?;
    save_to_path(&path, preferences)
}

fn save_to_path(
    path: &std::path::Path,
    preferences: UserPreferences,
) -> Result<UserPreferences, String> {
    preferences.validate_profiles()?;
    let parent = path
        .parent()
        .ok_or_else(|| "Preferences directory is unavailable.".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Preferences directory could not be created: {error}"))?;

    let normalized = preferences.normalized();
    let serialized = serde_json::to_vec_pretty(&normalized)
        .map_err(|error| format!("Preferences could not be serialized: {error}"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("Temporary preferences file could not be created: {error}"))?;
    temporary
        .write_all(&serialized)
        .map_err(|error| format!("Preferences could not be written: {error}"))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| format!("Preferences could not be flushed: {error}"))?;
    temporary
        .persist(path)
        .map_err(|error| format!("Preferences could not be saved: {}", error.error))?;

    Ok(normalized)
}

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(SETTINGS_FILE))
        .map_err(|error| format!("Preferences location is unavailable: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{load_from_path, save_to_path};
    use aircontrol_core::UserPreferences;

    #[test]
    fn missing_preferences_use_safe_defaults_and_saved_values_round_trip() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("preferences.json");

        let defaults = load_from_path(&path).expect("default preferences");
        assert_eq!(defaults, UserPreferences::default());

        let requested = UserPreferences {
            cursor_sensitivity: 1.7,
            reverse_scroll_direction: true,
            right_click_enabled: false,
            cursor_calibration: aircontrol_core::cursor::CursorCalibration {
                left: 0.12,
                top: 0.18,
                right: 0.88,
                bottom: 0.92,
            },
            auto_profile_enabled: true,
            app_profile_rules: vec![aircontrol_core::app_profile::AppProfileRule {
                executable: "editor.exe".to_owned(),
                profile_id: "presentation".to_owned(),
            }],
            ..UserPreferences::default()
        };
        let saved = save_to_path(&path, requested.clone()).expect("save preferences");
        let loaded = load_from_path(&path).expect("load saved preferences");
        let expected = requested.normalized();

        assert_eq!(saved, expected);
        assert_eq!(loaded, expected);
    }

    #[test]
    fn gesture_bindings_are_saved_and_restored_per_profile() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("preferences.json");
        let mut preferences = UserPreferences::default();
        preferences.gesture_studio.bindings[0].action =
            aircontrol_core::gesture_studio::GestureAction::Click;
        preferences.gesture_studio.bindings[0].sensitivity = 1.25;
        preferences.gesture_studio.bindings[0].cooldown_ms = 700;
        preferences.profiles[0].settings.gesture_studio = preferences.gesture_studio.clone();
        preferences.profiles[1].settings.gesture_studio.bindings[0].action =
            aircontrol_core::gesture_studio::GestureAction::Drag;

        save_to_path(&path, preferences).expect("save gesture settings");
        let loaded = load_from_path(&path).expect("load gesture settings");
        assert_eq!(
            loaded.gesture_studio.bindings[0].action,
            aircontrol_core::gesture_studio::GestureAction::Click
        );
        assert_eq!(loaded.gesture_studio.bindings[0].sensitivity, 1.25);
        assert_eq!(loaded.gesture_studio.bindings[0].cooldown_ms, 700);
        let presentation = loaded
            .select_profile("presentation")
            .expect("select presentation profile");
        assert_eq!(
            presentation.gesture_studio.bindings[0].action,
            aircontrol_core::gesture_studio::GestureAction::Drag
        );
    }

    #[test]
    fn preferences_saved_before_right_click_setting_use_enabled_default() {
        let old_preferences = br#"{
            "cursorSensitivity": 1.4,
            "cursorSmoothing": 0.3,
            "pinchSensitivity": 1.1,
            "scrollSensitivity": 1.2,
            "reverseScrollDirection": false,
            "cameraMirrored": true
        }"#;
        let parsed: UserPreferences = serde_json::from_slice(old_preferences)
            .expect("older preference data remains readable");

        assert!(parsed.right_click_enabled);
        assert!(!parsed.app_switch_enabled);
        assert!(!parsed.auto_profile_enabled);
        assert!(parsed.app_profile_rules.is_empty());
        assert_eq!(
            parsed.cursor_calibration,
            aircontrol_core::cursor::CursorCalibration::default()
        );
        let migrated = parsed.normalized();
        assert_eq!(migrated.active_profile_id, "productivity");
        assert_eq!(migrated.profiles.len(), 3);
        assert_eq!(migrated.profiles[0].settings.cursor_sensitivity, 1.4);
    }

    #[test]
    fn corrupted_preferences_return_an_error_without_silently_resetting() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("preferences.json");
        std::fs::write(&path, b"not valid json").expect("write fixture");

        assert!(load_from_path(&path).is_err());
        assert_eq!(
            std::fs::read(path).expect("preserved file"),
            b"not valid json"
        );
    }

    #[test]
    fn invalid_profiles_do_not_replace_the_last_saved_preferences() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("preferences.json");
        let existing = UserPreferences::default();
        save_to_path(&path, existing.clone()).expect("initial save");
        let mut invalid = existing;
        invalid.profiles[0].name = invalid.profiles[1].name.clone();

        assert!(save_to_path(&path, invalid).is_err());
        assert_eq!(
            load_from_path(&path).expect("preserved settings"),
            UserPreferences::default()
        );
    }

    #[test]
    fn malformed_saved_profiles_are_reported_without_overwriting_the_file() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("preferences.json");
        let malformed = br#"{"profiles":[]}"#;
        std::fs::write(&path, malformed).expect("write malformed profile fixture");

        assert!(load_from_path(&path).is_err());
        assert_eq!(std::fs::read(path).expect("preserved file"), malformed);
    }
}
