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
- Windows runner CI: passed on PR #2 (Rust formatting/tests/Clippy, frontend build/unit tests, Playwright, and Windows executable build).
- Windows Tauri executable, camera capture, and system input: not tested on this macOS host.

## In progress

- Phase 1 foundation: implementation and Windows CI validation are complete; PR #2 is ready to merge.

## Not implemented

- Camera capture and ONNX inference.
- Cursor, mouse gestures, window/app control, and Air Keyboard.
- Calibration, gesture customization, tray integration, and Windows packaging.
- Windows PC hardware validation.

## Next

1. Merge PR #2 after the successful Windows CI run.
2. Plan camera capture and hand inference as separately scoped issues after the foundation review.
