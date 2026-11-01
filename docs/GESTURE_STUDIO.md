# Gesture Studio

Gesture Studio provides profile-scoped gesture assignments, tuning, a live hand-landmark preview, current confidence/action feedback, recent state transitions, and a safe recognition test.

## Gesture bindings

Each profile stores one binding for index pinch, middle-finger pinch, two-finger scroll, and three-finger swipe. Each binding has a supported action, sensitivity from 0.6× to 1.4×, and cooldown from 100 to 2,000 ms. Unsupported and duplicate action assignments are rejected before persistence. Existing preference files receive the default bindings when loaded.

The live settings are copied into the safe-test recognizer when saved. Cursor and Window control consume the stored profile when the next explicit control session starts; the page says so beside Save. A profile change updates the runtime preferences through the existing settings command.

## Safe Testing Mode

The test mode can start only while the camera is running, a 21-point hand pose is detected, and all other control modes are disabled. Rust prepares the cursor controller in a non-dispatching state and rejects mode entry if a mouse button could still be held. While the mode is active, frame processing runs the recognizers only; it does not call the cursor, window, keyboard, or command-palette action dispatchers. Direct window actions are rejected by the Rust commands. Tracking loss, camera stop, inference failure, and explicit stop reset the test state.

The UI labels predictions as expected actions. They are not sent to Windows and do not prove that a real input action would be accepted by another application.

## Diagnostics

The panel displays camera and inference-derived tracking rates, measured inference duration, gesture-pipeline duration (including native dispatch while a system-control mode is active), tracking-loss transitions, and successful `SendInput` event count. CPU and working-set memory are reported only by the Windows process sampler. Unavailable measurements are shown as unavailable. Camera frames, landmarks, key contents, and typed text are not written to diagnostic logs.

## Validation boundaries

Unit and browser tests validate the settings model, conflict rules, disabled preview controls, and safe-mode recognition state machine. A Windows PC is still required to verify webcam delivery, landmark quality, actual profile activation, light/dark appearance in the native shell, and that system input remains absent during a hardware-backed safe test.
