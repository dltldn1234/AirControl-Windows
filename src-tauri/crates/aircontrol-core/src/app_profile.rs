use crate::{ControlProfile, UserPreferences};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppProfileRule {
    pub executable: String,
    pub profile_id: String,
}

pub fn normalize_executable_identity(value: &str) -> Result<String, String> {
    let file_name = value.trim().rsplit(['/', '\\']).next().unwrap_or_default();
    if file_name.is_empty()
        || file_name.chars().count() > 260
        || file_name.chars().any(char::is_control)
        || !file_name.to_ascii_lowercase().ends_with(".exe")
    {
        return Err("Enter a Windows executable name ending in .exe.".to_owned());
    }
    Ok(file_name.to_lowercase())
}

pub fn resolve_profile_id(
    automatic_profiles_enabled: bool,
    executable: &str,
    rules: &[AppProfileRule],
    profiles: &[ControlProfile],
) -> Option<String> {
    if !automatic_profiles_enabled {
        return None;
    }
    let identity = normalize_executable_identity(executable).ok()?;
    let rule = rules.iter().find(|rule| {
        normalize_executable_identity(&rule.executable).is_ok_and(|candidate| candidate == identity)
    })?;
    profiles
        .iter()
        .find(|profile| profile.id == rule.profile_id)
        .map(|profile| profile.id.clone())
}

pub fn preferences_for_executable(
    preferences: UserPreferences,
    executable: Option<&str>,
) -> UserPreferences {
    let Some(executable) = executable else {
        return preferences;
    };
    let Some(profile_id) = resolve_profile_id(
        preferences.auto_profile_enabled,
        executable,
        &preferences.app_profile_rules,
        &preferences.profiles,
    ) else {
        return preferences;
    };
    preferences
        .clone()
        .select_profile(&profile_id)
        .unwrap_or(preferences)
}

pub fn validate_app_profile_rules(preferences: &UserPreferences) -> Result<(), String> {
    if preferences.app_profile_rules.len() > 64 {
        return Err("At most 64 application profile rules can be saved.".to_owned());
    }
    let mut executables = HashSet::new();
    for rule in &preferences.app_profile_rules {
        let executable = normalize_executable_identity(&rule.executable)?;
        if !executables.insert(executable) {
            return Err("Each application can have only one automatic profile.".to_owned());
        }
        if !preferences
            .profiles
            .iter()
            .any(|profile| profile.id == rule.profile_id)
        {
            return Err("An application profile rule refers to a missing profile.".to_owned());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AppProfileRule, normalize_executable_identity, resolve_profile_id};
    use crate::{ControlProfile, ProfileSettings, UserPreferences};

    #[test]
    fn executable_identity_uses_only_a_case_normalized_basename() {
        assert_eq!(
            normalize_executable_identity(r"C:\\Program Files\\Editor\\Code.EXE"),
            Ok("code.exe".into())
        );
        assert_eq!(
            normalize_executable_identity("/opt/tools/editor.exe"),
            Ok("editor.exe".into())
        );
        assert!(normalize_executable_identity("document.txt").is_err());
        assert!(normalize_executable_identity("bad\nname.exe").is_err());
    }

    #[test]
    fn automatic_resolution_is_opt_in_and_falls_back_for_unmapped_or_missing_targets() {
        let profiles = ControlProfile::built_ins();
        let rules = vec![AppProfileRule {
            executable: "editor.exe".into(),
            profile_id: "presentation".into(),
        }];

        assert_eq!(
            resolve_profile_id(true, r"C:\\Apps\\EDITOR.EXE", &rules, &profiles),
            Some("presentation".into())
        );
        assert_eq!(
            resolve_profile_id(false, "editor.exe", &rules, &profiles),
            None
        );
        assert_eq!(
            resolve_profile_id(true, "unknown.exe", &rules, &profiles),
            None
        );

        let missing = vec![AppProfileRule {
            executable: "editor.exe".into(),
            profile_id: "deleted-profile".into(),
        }];
        assert_eq!(
            resolve_profile_id(true, "editor.exe", &missing, &profiles),
            None
        );
    }

    #[test]
    fn profile_rule_validation_rejects_duplicates_and_deleted_targets() {
        let duplicates = UserPreferences {
            app_profile_rules: vec![
                AppProfileRule {
                    executable: "Editor.exe".into(),
                    profile_id: "presentation".into(),
                },
                AppProfileRule {
                    executable: r"C:\\Tools\\editor.EXE".into(),
                    profile_id: "custom".into(),
                },
            ],
            ..UserPreferences::default()
        };
        assert!(duplicates.validate_profiles().is_err());

        let dangling = UserPreferences {
            app_profile_rules: vec![AppProfileRule {
                executable: "editor.exe".into(),
                profile_id: "deleted-profile".into(),
            }],
            ..UserPreferences::default()
        };
        assert!(dangling.validate_profiles().is_err());
    }

    #[test]
    fn new_profiles_keep_automatic_switching_disabled_by_default() {
        let preferences = UserPreferences::default();

        assert!(!preferences.auto_profile_enabled);
        assert!(preferences.app_profile_rules.is_empty());
        assert_eq!(ProfileSettings::default().cursor_sensitivity, 1.0);
    }

    #[test]
    fn executable_selection_applies_a_mapped_profile_and_preserves_fallbacks() {
        let preferences = UserPreferences {
            auto_profile_enabled: true,
            app_profile_rules: vec![AppProfileRule {
                executable: "editor.exe".into(),
                profile_id: "presentation".into(),
            }],
            ..UserPreferences::default()
        };
        let selected = super::preferences_for_executable(preferences.clone(), Some("EDITOR.EXE"));
        assert_eq!(selected.active_profile_id, "presentation");
        assert!(selected.app_switch_enabled);

        let fallback = super::preferences_for_executable(preferences.clone(), Some("unknown.exe"));
        assert_eq!(fallback, preferences);
        let unavailable = super::preferences_for_executable(preferences.clone(), None);
        assert_eq!(unavailable, preferences);
    }
}
