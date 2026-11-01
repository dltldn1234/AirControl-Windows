# Control Profiles

AirControl saves named control profiles inside its local `preferences.json` file. A profile contains cursor, pinch, window, scroll, mirroring, right-click, and app-switch preferences. Camera permission, camera selection, and the active system-control mode are not profile settings.

## Built-in presets

- **Productivity**: balanced cursor settings with app switching off.
- **Presentation**: steadier cursor smoothing with app switching enabled.
- **Custom**: a slightly higher cursor sensitivity starting point.

Selecting a profile only changes saved preferences. It never starts the camera or enables system input. Changes are picked up when the relevant control mode starts again.

## Personal profiles

Use **Save current as profile** to create a named copy of the current settings. Personal profiles can be renamed or deleted. Names must be unique and contain 1–32 characters. Built-in names cannot be changed; **Restore preset** returns the selected built-in profile to its supplied values.

Profile changes are written through the same atomic local settings save used by the rest of the app. Invalid profile data is rejected without replacing the last saved settings. Existing preference files without profile data migrate to the Productivity preset while retaining their current control values.

## Application-specific profiles

Application mappings are optional and disabled by default. Enable **Automatically select a profile**, enter a Windows executable name such as `notepad.exe`, and choose an existing profile. A full path may be entered, but only its lowercase executable basename is stored in `preferences.json`; window titles and document contents are never inspected or saved.

The selected mapping is resolved once when Cursor, Window, or Air Keyboard control starts. Window control identifies the explicitly selected target window. Cursor and Air Keyboard use the process returned by `GetForegroundWindow` at activation. The matched profile is applied to that control session and reported in runtime status; the user's saved default profile is not overwritten. If the executable cannot be queried, is protected, has no mapping, or refers to a deleted profile, the saved default profile is used. No foreground polling or security-boundary bypass is performed.

Deleting a custom profile also removes mappings that target it. At most 64 unique executable mappings can be stored. Matching is case-insensitive by executable basename, so applications that ship multiple executables should be mapped by the exact executable used for the selected window.
