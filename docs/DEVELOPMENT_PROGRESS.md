# Development Progress

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current `main` commit count: 28 after the ordinary merge of PR #8.
- Current issue: [#10 — Implement safe Windows window and app controls](https://github.com/dltldn1234/AirControl-Windows/issues/10)
- Current branch: `10-feat/windows-window-controls` (15 commits ahead of `main`; 41 commits reachable from this branch HEAD after the current progress update).
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

- Phase 5 cursor movement and pinch click/drag dispatch: merged through PR #8 after Windows CI passed; actual cursor behavior still needs a Windows PC check.
- Phase 6 scroll and right-click gestures: implemented in Issue #9; draft PR #11 is awaiting Windows CI.
- Phase 7 window and app control: implemented in Issue #10 branch; local checks pass, but Windows CI awaits the dependency PR merge.
- Phase 4 ONNX hand tracking: merged through PR #6 after successful Windows CI; actual webcam landmark quality still needs a Windows PC check.

## Not implemented

- Window/app control and Air Keyboard remain unimplemented. Window/app control planning is tracked in Issue #10.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Finish and merge PR #11 for Issue #9 after its Windows CI passes.
2. Push/open the Issue #10 PR once PR #11 is merged, then validate its Windows-only APIs in CI.
3. Validate camera, cursor, click, scroll, and window behavior on a Windows PC; this macOS host cannot provide that evidence.

## Follow-up (2026-11-01, Issue #9)

- Issue: [#9 — Add safe scroll and right-click gestures](https://github.com/dltldn1234/AirControl-Windows/issues/9). Work is isolated in `9-feat/scroll-right-click`, based on the Issue #7 branch so the cursor-control dependency is explicit.
- PR #8 for Issue #7 was merged normally at `66a144b110ffaeb9c6f0880940ccd563aa867575` after all Windows CI checks passed, including Windows executable compilation.
- Implemented in the Issue #9 worktree: normalized two-finger scroll recognition, reversed direction and sensitivity, per-frame wheel bounds and motion dead zone; thumb-middle short-tap right click, hysteresis, cooldown, and conflict resolution with left click; scroll suppresses cursor movement; tracking loss, stop, and teardown reset recognition and release both buttons.
- Added a persisted right-click enable preference. Deserialization defaults the new field for existing preference JSON files.
- Issue #9 local verification: Rust workspace tests 48 passed, Clippy with warnings denied passed, formatting passed, frontend typecheck passed, Vitest 5 passed, Vite production build passed, Playwright Chromium 2 passed. Windows `SendInput` and physical camera behavior still require Issue #9 Windows CI and a Windows PC.
- Five Issue #9 commits are pushed (`069fa4a`, `54302fc`, `b3bbadb`, `716e2bb`, `ed205b7`), and draft PR #11 targets `main`. Its Windows CI run `41281169144` is in progress.

## Follow-up (2026-11-01, Issue #10)

- Issue: [#10 — Implement safe Windows window and app controls](https://github.com/dltldn1234/AirControl-Windows/issues/10). Work is isolated in `10-feat/windows-window-controls`, stacked on the Issue #9 branch until PR #11 merges.
- Added pure tested window movement/resize geometry, size constraints, supported-window eligibility, and a pinch-to-move state machine with hysteresis and tracking-loss reset.
- Added Windows-only visible top-level window enumeration, filtering for hidden/owned/cloaked/tool/self windows, hex-string HWND transport to avoid JavaScript integer precision loss, and validated activation/move/resize/minimize/maximize commands. Protected/unsupported windows fail closed; foreground restrictions are reported without bypass.
- Added explicit camera-and-pose-gated Window mode. The user chooses a window and activates hand control; an index pinch moves only that target, while the distance between two tracked index fingertips scales it around its original center. Losing either hand ends resize immediately. Cursor control and window control cannot be enabled at the same time. Stop, tracking loss, inference errors, and camera shutdown end the mode.
- Added a working Window Control panel with list/selection, activate/minimize/maximize, explicit keyboard-button move/resize actions, and hand-move start/stop. Added a persisted window-movement sensitivity setting with a default for old preference files.
- Issue #10 local verification after two-hand resize: Rust workspace tests 59 passed, Clippy with warnings denied and formatting passed; frontend typecheck passed, Vitest 5 passed, Vite build passed, Playwright Chromium 3 passed. Windows API compilation and native controls await Windows CI after PR #11 merges; actual window behavior requires a Windows PC.
- Two-hand resize uses a normalized-distance recognizer with a dead zone, sensitivity clamp, scale clamp, original-rectangle sizing, and immediate reset on tracking loss. The runtime status and Window Control instructions now identify resize mode.
- Resize scale changes are limited to 0.12 per inference update to prevent abrupt size jumps if landmark positions shift. Core tests cover the cap.
- Issue #10 work remains stacked on Issue #9 and has not yet been opened as a PR. Resize commits: `ccc3976` (core recognition), `ecd0dbb` (native integration and status UI), and `19724f0` (resize smoothing).
- PR #11 Windows workflow run `41281169144` has passed every named test/build step, including the Windows executable build, but remains `in_progress` in the GitHub Actions post-job cache cleanup. The run has no final conclusion yet, so PR #11 is still unmerged and Issue #10 Windows CI has not started.
