import { describe, expect, it } from "vitest";
import {
  INITIAL_KEYBOARD_FEEDBACK,
  reduceKeyboardFeedback,
} from "./keyboardFeedback";

describe("Air Keyboard feedback", () => {
  it("clears the one-shot Shift indicator when hand landmarks are lost", () => {
    const armed = reduceKeyboardFeedback(INITIAL_KEYBOARD_FEEDBACK, {
      type: "keyAccepted",
      keyId: "shift",
    });

    expect(armed.shiftArmed).toBe(true);
    expect(
      reduceKeyboardFeedback(armed, {
        type: "trackingChanged",
        tracking: "lost",
      }),
    ).toEqual({ ...armed, shiftArmed: false });
  });

  it("keeps Shift armed while a hand pose remains detected", () => {
    const armed = { ...INITIAL_KEYBOARD_FEEDBACK, shiftArmed: true };
    expect(
      reduceKeyboardFeedback(armed, {
        type: "trackingChanged",
        tracking: "detected",
      }),
    ).toBe(armed);
  });

  it("clears one-shot Shift after an accepted non-modifier key", () => {
    const armed = { ...INITIAL_KEYBOARD_FEEDBACK, shiftArmed: true };
    expect(
      reduceKeyboardFeedback(armed, { type: "keyAccepted", keyId: "a" })
        .shiftArmed,
    ).toBe(false);
  });

  it("does not claim to know the system Caps Lock state", () => {
    const armed = { ...INITIAL_KEYBOARD_FEEDBACK, shiftArmed: true };
    expect(
      reduceKeyboardFeedback(armed, { type: "keyAccepted", keyId: "caps" }),
    ).toBe(armed);
  });

  it("labels the keys independently from the target application's IME mode", () => {
    expect(
      reduceKeyboardFeedback(INITIAL_KEYBOARD_FEEDBACK, {
        type: "keyAccepted",
        keyId: "layout",
      }),
    ).toEqual({ legend: "korean", shiftArmed: false });
  });

  it("preserves one-shot Shift when changing only the key legend", () => {
    expect(
      reduceKeyboardFeedback(
        { legend: "english", shiftArmed: true },
        { type: "keyAccepted", keyId: "layout" },
      ),
    ).toEqual({ legend: "korean", shiftArmed: true });
  });

  it("clears one-shot Shift but preserves key labels when keyboard mode stops", () => {
    expect(
      reduceKeyboardFeedback(
        { legend: "korean", shiftArmed: true },
        { type: "modeStopped" },
      ),
    ).toEqual({ legend: "korean", shiftArmed: false });
  });
});
