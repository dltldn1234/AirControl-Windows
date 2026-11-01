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
