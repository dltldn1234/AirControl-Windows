# Development Progress

## Current state (2026-11-01, Issue #23)

- Current `origin/main` baseline: 82 commits at `83e32af`; 119 meaningful commits remain to reach the user's 201-commit target. No filler commits will be created.
- Issue: [#23 — Add named control profiles and built-in presets](https://github.com/dltldn1234/AirControl-Windows/issues/23). Branch: `23-feat/control-profiles`, twelve commits ahead of the 82-commit baseline. PR creation is next after rebase onto the pending cursor-calibration merge.
- Added Rust profile models and validation, Productivity/Presentation/Custom presets, local atomic persistence within existing preferences, migration from older preference files, and Settings UI for selection, save-as, rename, delete, and preset restoration. Selecting a profile never activates the camera or system input.
- Local validation: Rust workspace 88 tests passed; rustfmt and Clippy with warnings denied passed; frontend typecheck, 23 Vitest tests, production build, and 6 Playwright tests passed. The latest profile lifecycle refactor passed typecheck and Vitest; production build and browser tests are being rerun.
- Issue #21 / PR #22 cursor calibration is still open. Latest Windows CI run `41289630039` passed Rust tests, Clippy, formatting, frontend build/tests, and browser tests; Windows executable build is still running. Merge only after the final job succeeds.
- Windows physical behavior for calibration and profile-driven control settings remains to be checked on a Windows PC.
- Next: finish PR #22 CI and merge normally if successful, rebase this profile branch, update this status with the new main count, then open PR #23 and wait for its Windows CI.

## Current state (2026-11-01, Issue #19)

- Directive baseline: 71 commits on `main` at `bb17403`; 130 additional meaningful commits would be needed to reach 201. This is a target, not a reason to create filler commits.
- Issue: [#19 — Implement gesture-driven app switching](https://github.com/dltldn1234/AirControl-Windows/issues/19). PR #20 is open from `19-feat/gesture-app-switching`; its Windows CI is pending. The branch has nine commits on the 71-commit baseline.
- Added normalized three-finger pose gating, horizontal swipe distance/velocity limits, cooldown, no-repeat-until-pose-release, tracking-loss reset, mirrored coordinate handling, and deterministic cycling among eligible visible windows.
- Window control settings include an opt-in app-switch toggle. Foreground activation continues to use `SetForegroundWindow`; policy denial is logged and ignored without bypass.
- Local verification: Rust workspace 73 tests passed, rustfmt and Clippy with warnings denied passed; frontend typecheck, 17 Vitest tests, Vite production build, and 5 Playwright tests passed. Windows CI and real application activation still need verification.
- Next: wait for PR #20 Windows CI, merge normally if it passes, and record the updated `main` commit count.

## Product roadmap toward 201+ meaningful commits

The 130-commit gap is expected to come from the following independently testable product work, not a per-commit quota. Scope each item as an Issue, then split implementation, safety behavior, UI wiring, tests, and documentation only when each part has a distinct purpose.

- Camera and tracking reliability: device reconnect, frame backpressure, inference timing, low-confidence handling, long-run resource release, lighting checks, and tracking recovery.
- Adaptive gesture engine: feature extraction, temporal confidence, pinch calibration, drag and scroll conflict resolution, configurable cooldowns, gesture profiles, and replayable landmark fixtures.
- Cursor and spatial control: monitor/DPI mapping tests, cursor filter tuning, window-boundary behavior, safe target refresh, resize limits, activation policy failures, and app-switching regression coverage.
- Air Keyboard and IME: non-activating overlay focus checks, key repeat, modifier release, target IME guidance, language/layout selection semantics, hit-test scaling, and Windows manual-test procedures.
- Calibration and profiles: calibration persistence/versioning, onboarding, reset/restore, per-profile settings, profile import/export validation, and automatic-profile safety.
- Product UI: dashboard health states, camera selection, gesture configuration, calibration flow, performance details, command palette, tray menu, keyboard navigation, accessibility names, and responsive layouts.
- Reliability, QA, and release: native integration seams, deterministic tests, Playwright workflows, Windows CI artifact checks, installer/signing readiness documentation, upgrade behavior, diagnostics, privacy review, and troubleshooting coverage.

Only changes that improve behavior, safety, maintainability, or verifiable quality will be committed. If the real backlog cannot produce 201 commits without padding, report the remaining gap instead of manufacturing work.

## Previous state (Issue #17)

- Issue: [#17 — Keep tracked hand identities stable across camera frames](https://github.com/dltldn1234/AirControl-Windows/issues/17); PR #18 merged normally at `bb17403` after Windows CI run [41288181630](https://github.com/dltldn1234/AirControl-Windows/actions/runs/41288181630) passed.
- Added a two-track wrist-center associator with bounded nearest-neighbor matching, short velocity prediction, two-frame loss tolerance, and reset behavior. Camera routing now binds landmarks to stable track slots instead of detector confidence order, and falls back to the remaining live hand without reusing stale landmarks.
- Local verification: Rust workspace 75 tests passed, rustfmt passed, Clippy with warnings denied passed. Manual camera crossing/reacquisition, cursor hand continuity, and window resize checks remain Windows PC checks.
- Issue #17 contributed four branch commits on the 66-commit baseline; merge commit brought `main` to 71 commits.

## Previous state (Issue #15)

- Issue: [#15 — Keep Air Keyboard modifier feedback accurate after tracking loss](https://github.com/dltldn1234/AirControl-Windows/issues/15); PR #16 merged normally at `9324912` after Windows CI run [41287447837](https://github.com/dltldn1234/AirControl-Windows/actions/runs/41287447837) passed.
- Removed AirControl's local Caps Lock state because it cannot represent the target app's global Windows toggle state. The key still emits the Caps Lock virtual key.
- Shift feedback now follows native keyboard session transitions and clears when tracking is lost; accepted keys, layout selection, and mode stop share a tested feedback reducer.
- English/Korean is labeled as a keycap legend selection. UI copy states that AirControl does not inspect the foreground application's IME state and directs users to that application's indicator.
- Local verification: Rust workspace 68 tests passed, rustfmt and Clippy passed; frontend typecheck, 16 Vitest tests, Vite production build, and 4 Playwright tests passed. Windows CI passed including Windows executable build.
- Manual Windows overlay/focus, global Caps Lock, and Korean IME behavior remain unverified on physical Windows hardware.
- Issue #15 contributed six branch commits on the 59-commit baseline; merge commit brought `main` to 66 commits.

## Previous state (2026-11-01, Issue #13)

- `main` baseline: 50 commits at `f16f47cdf3af38948ee155e99c5e98185fd0e7fc` (PR #12 merge); Issue #13 branch `13-feat/air-keyboard` began from that head.
- Issue: [#13 — Implement safe Air Keyboard input](https://github.com/dltldn1234/AirControl-Windows/issues/13).
- Air Keyboard implementation is complete on this branch: non-focusable floating Tauri surface, opt-in camera-gated mode, hand cursor movement with a short-pinch-only click recognizer restricted to the live keyboard window bounds, Rust allow-listed key events, English QWERTY and Korean Dubeolsik legends, and Shift/Caps/IME/editing controls.
- Key-event dispatch is denied outside Air Keyboard mode and when the floating surface is hidden. No typed content is stored or logged. Modifier state resets on tracking loss, camera stop, mode stop, and inference/input errors.
- Local checks on macOS: Rust workspace tests 67 passed, rustfmt passed, Clippy with warnings denied passed, frontend typecheck passed, Vitest 9 passed, Vite production build passed, Playwright Chromium 4 passed.
- Windows CI run [41284074048](https://github.com/dltldn1234/AirControl-Windows/actions/runs/41284074048) passed on the Windows runner, including formatting, Rust tests, Clippy, frontend checks/tests, Playwright, and Windows Tauri executable compilation.
- The non-activating overlay, target-app focus retention, actual `SendInput` behavior, webcam interaction, and Korean IME composition still require manual testing on a Windows PC; automatic tests do not establish those behaviors.
- At branch start, `main` had 50 commits. Seven scoped Air Keyboard implementation, safety, and documentation commits are present on this branch; current branch count is 57 including the base history.

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current `main` commit count: 34 after the ordinary merge of PR #11.
- Current issue: [#10 — Implement safe Windows window and app controls](https://github.com/dltldn1234/AirControl-Windows/issues/10)
- Current branch: `10-feat/windows-window-controls` (14 commits ahead of `main`; 48 commits reachable from this branch HEAD after the current progress update).
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

## Current outstanding work

- Phase 5 cursor movement and pinch click/drag dispatch: merged through PR #8 after Windows CI passed; actual cursor behavior still needs a Windows PC check.
- Phase 6 scroll and right-click gestures: implemented and merged through PR #11 after all Windows CI checks passed.
- Phase 7 window control: merged through PR #12; real window behavior needs a Windows PC.
- Phase 4 ONNX hand tracking: merged through PR #6 after successful Windows CI; actual webcam landmark quality still needs a Windows PC check.

## Not implemented or awaiting hardware verification

- Gesture-driven app switching is in progress in Issue #19; the current main only activates a user-selected eligible window.
- Calibration, gesture customization, tray integration, and Windows packaging remain unimplemented.
- Air Keyboard physical IME/focus behavior and hand tracking quality need Windows PC validation.
- Windows PC hardware validation.

## Next

1. Finish and submit Issue #19 for Windows CI review.
2. Continue with calibration and gesture customization as separate issues.
3. Validate camera, cursor, click, scroll, resize, Air Keyboard IME/focus, and tracking continuity on a Windows PC; this macOS host cannot provide that evidence.

## Follow-up (2026-11-01, Issue #9)

- Issue: [#9 — Add safe scroll and right-click gestures](https://github.com/dltldn1234/AirControl-Windows/issues/9). Work is isolated in `9-feat/scroll-right-click`, based on the Issue #7 branch so the cursor-control dependency is explicit.
- PR #8 for Issue #7 was merged normally at `66a144b110ffaeb9c6f0880940ccd563aa867575` after all Windows CI checks passed, including Windows executable compilation.
- Implemented in the Issue #9 worktree: normalized two-finger scroll recognition, reversed direction and sensitivity, per-frame wheel bounds and motion dead zone; thumb-middle short-tap right click, hysteresis, cooldown, and conflict resolution with left click; scroll suppresses cursor movement; tracking loss, stop, and teardown reset recognition and release both buttons.
- Added a persisted right-click enable preference. Deserialization defaults the new field for existing preference JSON files.
- Issue #9 local verification: Rust workspace tests 48 passed, Clippy with warnings denied passed, formatting passed, frontend typecheck passed, Vitest 5 passed, Vite production build passed, Playwright Chromium 2 passed. Windows CI passed on PR #11, including Windows desktop compilation; physical camera and input behavior remain a Windows PC check.
- PR #11 for Issue #9 was merged with ordinary merge commit `969b0879cd4b2646183d209b1aeca74720c2000b` after the Windows CI run `41281169144` completed successfully.

## Follow-up (2026-11-01, Issue #10)

- Issue: [#10 — Implement safe Windows window and app controls](https://github.com/dltldn1234/AirControl-Windows/issues/10). Work is isolated in `10-feat/windows-window-controls`, stacked on the Issue #9 branch until PR #11 merges.
- Added pure tested window movement/resize geometry, size constraints, supported-window eligibility, and a pinch-to-move state machine with hysteresis and tracking-loss reset.
- Added Windows-only visible top-level window enumeration, filtering for hidden/owned/cloaked/tool/self windows, hex-string HWND transport to avoid JavaScript integer precision loss, and validated activation/move/resize/minimize/maximize commands. Protected/unsupported windows fail closed; foreground restrictions are reported without bypass.
- Added explicit camera-and-pose-gated Window mode. The user chooses a window and activates hand control; an index pinch moves only that target, while the distance between two tracked index fingertips scales it around its original center. Losing either hand ends resize immediately. Cursor control and window control cannot be enabled at the same time. Stop, tracking loss, inference errors, and camera shutdown end the mode.
- Added a working Window Control panel with list/selection, activate/minimize/maximize, explicit keyboard-button move/resize actions, and hand-move start/stop. Added a persisted window-movement sensitivity setting with a default for old preference files.
- Issue #10 local verification after two-hand resize: Rust workspace tests 59 passed, Clippy with warnings denied and formatting passed; frontend typecheck passed, Vitest 5 passed, Vite build passed, Playwright Chromium 3 passed. Windows API compilation awaits the Issue #10 PR checks; actual window behavior requires a Windows PC.
- Two-hand resize uses a normalized-distance recognizer with a dead zone, sensitivity clamp, scale clamp, original-rectangle sizing, and immediate reset on tracking loss. The runtime status and Window Control instructions now identify resize mode.
- Resize scale changes are limited to 0.12 per inference update to prevent abrupt size jumps if landmark positions shift. Core tests cover the cap.
- Issue #10 PR #12 is open as a draft against `main`, after a normal merge of the newly updated base; its diff contains only window-control changes.
- While Window mode is active, the UI now locks target selection, refresh, and direct window actions, keeping the displayed target consistent with the native controller target.
- PR #12 Windows CI is running for the latest pushed revision; its result must pass before merge.
- Resize commits: `ccc3976` (core recognition), `ecd0dbb` (native integration and status UI), and `19724f0` (resize smoothing).
