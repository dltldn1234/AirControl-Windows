# Development Progress

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current `main` commit count: 14 after the ordinary merge of PR #6. Issue #5 contributed 4 feature commits and a merge commit.
- Current issue: [#7 — Add safe Windows cursor movement and click dispatch](https://github.com/dltldn1234/AirControl-Windows/issues/7)
- Current branch: `7-feat/windows-cursor-control` (12 commits ahead of `main` in PR #8; 25 commits reachable from this branch HEAD).
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

- PR #6 for Issue #5 was merged with ordinary merge commit `382577f145803f890f040cad7a4044be729feecb`; `main` now has 14 commits. Its Windows executable build, Rust checks, frontend checks, and Playwright all passed in Windows CI.
- Issue #7, `7-feat/windows-cursor-control`, is pushed with 12 commits ahead of `main` and a draft PR #8. It is based on PR #6's hand-tracking head; the PR #6 merge commit remains on `main`. The separate worktree preserved the original checkout's untracked `README 2.md` and `src/App 2.tsx`.
- Added pure virtual-desktop coordinate mapping for negative monitor origins, mirroring, bounds, sensitivity, and invalid geometry.
- Added a hysteretic pinch state machine: short pinch clicks, sustained pinch starts drag, release or tracking loss ends drag, and cooldown avoids immediate rearming.
- Added a One Euro cursor filter with an adjustable cutoff driven by the saved smoothing preference and a small dead zone; the filter resets after tracking loss.
- Connected opt-in Windows `SendInput` cursor movement and left-button events to the live 21-point hand stream; activation is gated on running camera and confident hand-pose tracking. Camera stop, inference failure, tracking loss, and controller teardown release any held button.
- Added a real dashboard control toggle and distinguished palm-only detection from full 21-landmark tracking. Browser-only mode keeps cursor control disabled.
- Runtime gesture status now follows click/drag transitions and returns to cursor mode after release.
- Issue #7 local verification: Rust workspace tests 38 passed, Clippy with warnings denied passed, formatting passed, TypeScript typecheck passed, Vitest 5 passed (including native camera/pose activation gating), Vite production build passed, Playwright Chromium 2 passed.
- Issue #7 Windows native compilation and actual cursor behavior remain pending Windows CI and Windows PC validation; this macOS host cannot exercise `SendInput` or a Windows camera.

## In progress

- Phase 5 cursor movement and pinch click/drag dispatch: implemented and pushed in Issue #7; awaiting PR Windows CI and device validation.
- Phase 4 ONNX hand tracking: merged through PR #6 after successful Windows CI; actual webcam landmark quality still needs a Windows PC check.

## Not implemented

- Window/app control, scrolling and right-click gestures, and Air Keyboard. Cursor movement, left-click, and drag dispatch are in the unmerged Issue #7 branch above.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Merge PR #8 after the Windows browser tests and Windows desktop executable build both pass.
2. Rebase/retain Issue #9 on the merged cursor-control history, then open its draft PR and wait for Windows CI.
3. Validate camera, cursor, click, and scroll behavior on a Windows PC; this macOS host cannot provide that evidence.

## Follow-up (2026-11-01, Issue #9)

- Issue: [#9 — Add safe scroll and right-click gestures](https://github.com/dltldn1234/AirControl-Windows/issues/9). Work is isolated in `9-feat/scroll-right-click`, based on the Issue #7 branch so the cursor-control dependency is explicit.
- PR #8 for Issue #7 remains open and draft while its Windows runner completes. Rust tests and Clippy, frontend typecheck/unit/build/browser tests, and the Windows executable build are part of that runner; no merge is performed before all checks pass.
- Implemented in the Issue #9 worktree: normalized two-finger scroll recognition, reversed direction and sensitivity, per-frame wheel bounds and motion dead zone; thumb-middle short-tap right click, hysteresis, cooldown, and conflict resolution with left click; scroll suppresses cursor movement; tracking loss, stop, and teardown reset recognition and release both buttons.
- Added a persisted right-click enable preference. Deserialization defaults the new field for existing preference JSON files.
- Issue #9 local verification: Rust workspace tests 48 passed, Clippy with warnings denied passed, formatting passed, frontend typecheck passed, Vitest 5 passed, Vite production build passed, Playwright Chromium 2 passed. Windows `SendInput` and physical camera behavior still require Windows CI and a Windows PC.
- Three Issue #9 commits are recorded locally (`069fa4a`, `54302fc`, `b3bbadb`); they have not been pushed or opened as a PR. The branch awaits Issue #7 merge so the new PR can target `main` without including cursor-control commits twice.
