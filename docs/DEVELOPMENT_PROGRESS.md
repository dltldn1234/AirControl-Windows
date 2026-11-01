# Development Progress

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current `main` commit count: 9; current Issue #5 branch contains 4 feature commits after the live 21-point update (13 commits reachable at HEAD).
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
- Implemented live palm detector input preprocessing, 2016-anchor decoding, confidence filtering, non-maximum suppression, camera-frame inference, hand-tracking status updates, and mirrored palm bounding-box/landmark overlays.
- Implemented palm-oriented hand crop rotation/padding, 21-point ONNX inference and confidence filtering, source-image coordinate mapping, and mirrored finger skeleton rendering.
- Updated the browser camera smoke test to check the precise finger-landmark inference status shown while browsing outside the native desktop app.
- Stop retrying the model session after a deterministic inference error while preserving camera preview; this avoids repeated error logging and wasted inference time.

## Verification

- macOS Rust workspace tests: 19 passed (3 core, 16 Tauri library), including bundled ONNX session loading, blank-frame palm inference, detector preprocessing, landmark crop inference, coordinate mapping, rotation inversion, and shape validation.
- Rust formatting: passed.
- Clippy with warnings denied: passed.
- Frontend typecheck and production build: passed.
- Vitest: 4 passed.
- Playwright Chromium: 2 passed; camera page distinguishes loaded model assets from inactive live inference.
- Windows runner CI: passed on PR #2 (Rust formatting/tests/Clippy, frontend build/unit tests, Playwright, and Windows executable build).
- Windows runner CI for PR #4: passed, including the Windows Tauri executable build.
- Local Rust formatting, tests (19), and Clippy with warnings denied: passed after adding 21-point inference, a real ONNX crop invocation, and rotated coordinate mapping coverage.
- Local frontend typecheck and production build: passed; Vitest: 4 passed; Playwright Chromium: 2 passed after refreshing the camera page's status copy.
- macOS Tauri release build after adding ONNX Runtime: failed at link time because the prebuilt ORT CoreML objects target macOS 13.4 while the host link target is macOS 11.0 (`___isPlatformVersionAtLeast` unresolved). Temporarily setting `MACOSX_DEPLOYMENT_TARGET=13.4` did not override the Rust target linker flag. The product target is Windows; Windows CI is required to verify its native build.
- Windows camera capture and system input: not tested on this macOS host.
- Windows CI for the previously pushed model-loading revision was still running at last check. The current palm and landmark pipeline requires its own Windows runner verification after push.

## Current follow-up (2026-11-01)

- Main still has 9 commits while PR #6 (Issue #5, ONNX hand tracking) awaits the Windows executable build step; formatting, Rust tests, Clippy, frontend typecheck/build, Vitest, and Playwright steps passed on its Windows run.
- Issue #7, `7-feat/windows-cursor-control`, is being implemented in a separate worktree based on PR #6's head so the original checkout and its untracked `README 2.md` and `src/App 2.tsx` remain untouched.
- Added pure virtual-desktop coordinate mapping for negative monitor origins, mirroring, bounds, sensitivity, and invalid geometry.
- Added a hysteretic pinch state machine: short pinch clicks, sustained pinch starts drag, release or tracking loss ends drag, and cooldown avoids immediate rearming.
- Connected opt-in Windows `SendInput` cursor movement and left-button events to the live 21-point hand stream; activation is gated on running camera and confident hand-pose tracking. Camera stop, inference failure, tracking loss, and controller teardown release any held button.
- Added a real dashboard control toggle and distinguished palm-only detection from full 21-landmark tracking. Browser-only mode keeps cursor control disabled.
- Issue #7 local verification: Rust workspace tests 34 passed, Clippy with warnings denied passed, formatting passed, TypeScript typecheck passed, Vitest 4 passed, Vite production build passed, Playwright Chromium 2 passed.
- Issue #7 Windows native compilation and actual cursor behavior remain pending Windows CI and Windows PC validation; this macOS host cannot exercise `SendInput` or a Windows camera.

## In progress

- Phase 5 cursor movement and pinch click/drag dispatch: implemented locally in Issue #7 worktree; awaiting commit, PR, Windows CI, and device validation.
- Phase 4 ONNX hand tracking: model inference and visual overlay implemented in Issue #5 branch; awaiting the Windows executable build result in PR #6.

## Not implemented

- Cursor, mouse gestures, window/app control, and Air Keyboard.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Commit the live inference change, update Issue #5's draft PR, and verify its Windows runner result.
2. Validate actual palm and landmark locations using a Windows webcam; tune crop transform or confidence thresholds if needed.
