# AirControl for Windows

AirControl is a local-first Windows desktop application for controlling a PC with hand gestures. The Windows edition is built with Tauri 2, a Rust core, and a React + TypeScript interface.

> **Development status:** the desktop shell, runtime status bridge, local preference storage, dashboard, opt-in native camera enumeration/preview, local ONNX palm plus 21-point hand landmark inference, cursor/mouse gestures, window control, Air Keyboard, gesture-driven app switching, and cursor calibration are implemented. Named control profiles are in progress in Issue #23. Camera, keyboard IME, window activation, and physical cursor behavior still need Windows PC validation.

## Technology

- Rust core and Tauri 2 commands
- React, TypeScript, Vite, Tailwind CSS, and Zustand
- Rust unit tests, Vitest, and Playwright
- Windows native builds run in GitHub Actions on `windows-latest`

## Requirements

- Windows 10 or 11 x64 for native app development and runtime validation
- Rust stable MSVC toolchain
- Node.js 22 LTS and npm
- Microsoft C++ Build Tools with the Desktop development with C++ workload
- Microsoft Edge WebView2 Runtime

Tauri uses WebView2 on Windows; Windows builds also require the Microsoft C++ build tools. See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

```powershell
npm ci
npm run tauri dev
```

Useful checks:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --workspace --manifest-path src-tauri/Cargo.toml
cargo clippy --workspace --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings
npm run build
npm test
npm run test:e2e
```

## Architecture

- `src-tauri/crates/aircontrol-core`: platform-independent runtime states, safety gates, and preference models.
- `src-tauri/src`: Tauri commands and Windows desktop integration services.
- `src`: React screens, presentation state, and Tauri command calls.
- `e2e`: browser-level UI checks; these do not replace testing the Windows WebView2 app.

React does not generate operating-system input. Cursor, window, and keyboard control belongs in Rust services behind explicit readiness and user activation gates.
Named control profiles and built-in presets are stored locally; see [control profiles](docs/CONTROL_PROFILES.md).

## Privacy and safety

- Camera frames and typed text must remain local and must not be logged.
- Camera access and all system input are opt-in.
- Cursor control requires an explicit user action and the required Windows permissions.
- Camera access is requested only when the user searches for devices; capture is stopped when the user stops the preview or the app exits.
- Camera frames are resized and kept in memory for the live preview. They are not saved or sent to a server.
- ONNX model weights are bundled for local processing; see [model notices](docs/MODEL_NOTICES.md). Palm detection and 21-point landmark inference run locally on sampled camera preview frames.
- Native system input is gated behind explicit control activation and Windows desktop permissions; browser preview never sends system input.

## License

This repository is private and currently has an all-rights-reserved notice. No public redistribution license has been granted.

## Validation boundary

Frontend and portable Rust-core checks can run on macOS. A successful macOS check does not validate the Windows Tauri executable, WebView2, Windows camera capture, or native input. Windows build/package checks run in the repository's GitHub Actions workflow; camera and system-control behavior requires manual testing on a Windows PC.
