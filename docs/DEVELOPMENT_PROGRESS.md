# Development Progress

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current `main` commit count: 9; current Issue #5 branch contains 2 additional commits (11 commits reachable at current HEAD).
- Current issue: [#5 — Add local ONNX hand detection and landmark tracking](https://github.com/dltldn1234/AirControl-Windows/issues/5)
- Current branch: `5-feat/onnx-hand-tracking`
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
- Merged PR #4 normally after all Windows CI checks succeeded; repository main now has 9 commits.
- Added the OpenCV Zoo MediaPipe palm and hand landmark ONNX assets, Apache-2.0 notice, and pinned SHA-256 values.
- Added ONNX Runtime session initialization with telemetry disabled and bundled model validation tests.

## Verification

- macOS Rust workspace tests: 10 passed (3 core, 7 Tauri library), including loading both bundled ONNX sessions.
- Rust formatting: passed.
- Clippy with warnings denied: passed.
- Frontend typecheck and production build: passed.
- Vitest: 4 passed.
- Playwright Chromium: 2 passed; camera page distinguishes loaded model assets from inactive live inference.
- Windows runner CI: passed on PR #2 (Rust formatting/tests/Clippy, frontend build/unit tests, Playwright, and Windows executable build).
- Windows runner CI for PR #4: passed, including the Windows Tauri executable build.
- Local formatting, Rust tests (10), Clippy, frontend build/tests (4), and Playwright (2) for ONNX model session loading: passed.
- macOS Tauri release build after adding ONNX Runtime: failed at link time because the prebuilt ORT CoreML objects target macOS 13.4 while the host link target is macOS 11.0 (`___isPlatformVersionAtLeast` unresolved). Temporarily setting `MACOSX_DEPLOYMENT_TARGET=13.4` did not override the Rust target linker flag. The product target is Windows; Windows CI is required to verify its native build.
- Windows camera capture and system input: not tested on this macOS host.
- PR #4 Windows CI: passed, including ONNX-independent current camera build; model-session tests added for Issue #5 and pending execution.

## In progress

- Phase 4 ONNX hand tracking: licensed model assets load into local ONNX Runtime sessions when camera capture starts, and model status is shown. Frame inference/preprocessing, palm decoding, landmark postprocessing/overlay, and Windows CI for Issue #5 remain.

## Not implemented

- Palm detection and 21-landmark inference are not yet wired into camera frames.
- Cursor, mouse gestures, window/app control, and Air Keyboard.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Finish local and Windows CI validation for Issue #3, then merge its PR normally.
2. Plan ONNX hand inference as a separate issue after native camera capture is validated.
