import type { RuntimeSnapshot } from "./domain/runtimeStatus";

export type KeyboardLegend = "english" | "korean";

export type KeyboardFeedback = {
  legend: KeyboardLegend;
  shiftArmed: boolean;
};

export type KeyboardFeedbackEvent =
  | { type: "keyAccepted"; keyId: string }
  | { type: "trackingChanged"; tracking: RuntimeSnapshot["handTracking"] }
  | { type: "modeStopped" };

export const INITIAL_KEYBOARD_FEEDBACK: KeyboardFeedback = {
  legend: "english",
  shiftArmed: false,
};

export function reduceKeyboardFeedback(
  state: KeyboardFeedback,
  event: KeyboardFeedbackEvent,
): KeyboardFeedback {
  switch (event.type) {
    case "keyAccepted":
      if (event.keyId === "shift") {
        return { ...state, shiftArmed: !state.shiftArmed };
      }
      if (event.keyId === "layout") {
        return {
          legend: state.legend === "english" ? "korean" : "english",
          shiftArmed: state.shiftArmed,
        };
      }
      return event.keyId === "caps" ? state : { ...state, shiftArmed: false };
    case "trackingChanged":
      return event.tracking === "detected" || !state.shiftArmed
        ? state
        : { ...state, shiftArmed: false };
    case "modeStopped":
      return state.shiftArmed ? { ...state, shiftArmed: false } : state;
  }
}
