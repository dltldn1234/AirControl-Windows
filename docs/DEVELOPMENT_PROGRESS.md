# Development Progress

## Repository baseline

- Baseline commit count: 0 (empty repository at project start)
- Current issue: [#1 — Initialize the Tauri 2, Rust, React, and Windows CI foundation](https://github.com/dltldn1234/AirControl-Windows/issues/1)
- Current branch: `1-chore/tauri-phase1-foundation`
- Active stack: Rust, Tauri 2, React, TypeScript, Vite, Tailwind CSS

## Completed

- Created the private GitHub repository and connected the local checkout.
- Added the Tauri 2 desktop shell and React dashboard/navigation.
- Added a Rust core crate with runtime types, safe default state, and input readiness gating.
- Added Tauri commands for runtime status and local preferences.
- Added validated preference persistence with corruption-preserving error behavior.
- Added Windows GitHub Actions checks for formatting, tests, Clippy, frontend build/tests, browser tests, and a Windows executable build.
- Added Vitest and Playwright smoke coverage.

## Verification

- macOS Rust workspace tests: 5 passed.
- Rust formatting: passed.
- Clippy with warnings denied: passed.
- Frontend typecheck and production build: passed.
- Vitest: 4 passed.
- Playwright Chromium: 2 passed.
- Windows runner CI: not run yet; requires the branch to be pushed and a PR opened.
- Windows Tauri executable, camera capture, and system input: not tested on this macOS host.

## In progress

- Phase 1 foundation: local implementation is ready for Windows CI validation and review.
- Establishing the first `main` commit so the feature branch can be reviewed through a pull request.

## Not implemented

- Camera capture and ONNX inference.
- Cursor, mouse gestures, window/app control, and Air Keyboard.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Run the native Tauri host build where supported and verify the Windows workflow through GitHub Actions.
2. Fix Windows CI failures before merging.
3. Plan camera capture and hand inference as separately scoped issues after the foundation review.
