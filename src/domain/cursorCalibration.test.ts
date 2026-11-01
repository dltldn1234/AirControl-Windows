import { describe, expect, it } from "vitest";
import { DEFAULT_CURSOR_CALIBRATION, moveCalibrationEdge } from "./cursorCalibration";

describe("cursor calibration bounds", () => {
  it("keeps at least 15 percent of camera range on both axes", () => {
    expect(moveCalibrationEdge(DEFAULT_CURSOR_CALIBRATION, "left", 0.95)).toEqual({
      ...DEFAULT_CURSOR_CALIBRATION,
      left: 0.85,
    });
    expect(moveCalibrationEdge(DEFAULT_CURSOR_CALIBRATION, "bottom", 0.1)).toEqual({
      ...DEFAULT_CURSOR_CALIBRATION,
      bottom: 0.15,
    });
  });

  it("clamps non-finite and out-of-range slider values safely", () => {
    expect(moveCalibrationEdge(DEFAULT_CURSOR_CALIBRATION, "right", 4).right).toBe(1);
    expect(moveCalibrationEdge(DEFAULT_CURSOR_CALIBRATION, "top", Number.NaN).top).toBe(0);
  });
});
