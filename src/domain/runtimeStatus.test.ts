import { describe, expect, it } from "vitest";
import { cameraLabel, canEnableCursorControl, gestureLabel, trackingLabel } from "./runtimeStatus";
import type { RuntimeSnapshot } from "./runtimeStatus";

describe("runtime status labels", () => {
  it("explains camera lifecycle states without implying a live connection", () => {
    expect(cameraLabel("running")).toBe("Connected");
    expect(cameraLabel("enumerating")).toBe("Searching");
    expect(cameraLabel("permissionRequired")).toBe("Permission needed");
    expect(cameraLabel("unavailable")).toBe("Not connected");
  });

  it("presents tracking loss and gesture names clearly", () => {
    expect(trackingLabel("lost")).toBe("Tracking lost");
    expect(trackingLabel("modelReady")).toBe("Model loaded");
    expect(trackingLabel("palmDetected")).toBe("Palm detected");
    expect(trackingLabel("detected")).toBe("21 landmarks");
    expect(gestureLabel("rightClick")).toBe("Right click");
    expect(gestureLabel("windowMove")).toBe("Move window");
    expect(gestureLabel("windowResize")).toBe("Resize window");
    expect(gestureLabel("appSwitch")).toBe("Switch application");
    expect(gestureLabel("none")).toBe("None");
  });

  it("enables system cursor input only after native camera and pose readiness", () => {
    const ready: RuntimeSnapshot = {
      camera: "running",
      handTracking: "detected",
      controlMode: "disabled",
      activeGesture: "none",
      framesPerSecond: 10,
    };
    expect(canEnableCursorControl(ready, "connected")).toBe(true);
    expect(canEnableCursorControl(ready, "preview")).toBe(false);
    expect(canEnableCursorControl({ ...ready, camera: "stopped" }, "connected")).toBe(false);
    expect(canEnableCursorControl({ ...ready, handTracking: "palmDetected" }, "connected")).toBe(false);
    expect(canEnableCursorControl({ ...ready, controlMode: "cursor" }, "connected")).toBe(false);
    expect(canEnableCursorControl(null, "connected")).toBe(false);
  });
});
