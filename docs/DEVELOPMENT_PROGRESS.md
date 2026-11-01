# Development Progress

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current issue: [#3 — Add native camera capture and preview](https://github.com/dltldn1234/AirControl-Windows/issues/3)
- Current branch: `3-feat/camera-preview`
- Active stack: Rust, Tauri 2, React, TypeScript, Vite, Tailwind CSS

## Completed

- Created the private GitHub repository and connected the local checkout.
- Added the Tauri 2 desktop shell and React dashboard/navigation.
- Added a Rust core crate with runtime types, safe default state, and input readiness gating.
- Added Tauri commands for runtime status and local preferences.
- Added validated preference persistence with corruption-preserving error behavior.
- Added Windows GitHub Actions checks for formatting, tests, Clippy, frontend build/tests, browser tests, and a Windows executable build.
- Added Vitest and Playwright smoke coverage.
- Added opt-in native camera enumeration and preview capture using Nokhwa; frames are downscaled and sent locally to the UI without persistence.
- Added a camera source picker, permission/error/empty states, live preview controls, and preview-only browser messaging.
- Added macOS camera permission usage text for local development.

## Verification

- macOS Rust workspace tests: 8 passed (3 core, 5 Tauri library).
- Rust formatting: passed.
- Clippy with warnings denied: passed.
- Frontend typecheck and production build: passed.
- Vitest: 4 passed.
- Playwright Chromium: 2 passed after updating the camera-page expectation.
- Windows runner CI: passed on PR #2 (Rust formatting/tests/Clippy, frontend build/unit tests, Playwright, and Windows executable build). Issue #3 Windows CI pending.
- macOS Tauri native release build: passed; app executable generated.
- Windows camera capture and system input: not tested on this macOS host.

## In progress

- Phase 2 camera capture/preview: native service and UI are implemented; local checks passed and Issue #3 Windows CI remains to verify.

## Not implemented

- ONNX hand inference and landmarks.
- Cursor, mouse gestures, window/app control, and Air Keyboard.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Finish local and Windows CI validation for Issue #3, then merge its PR normally.
2. Plan ONNX hand inference as a separate issue after native camera capture is validated.
