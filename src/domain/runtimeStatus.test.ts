import { describe, expect, it } from "vitest";
import { cameraLabel, gestureLabel, trackingLabel } from "./runtimeStatus";

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
    expect(gestureLabel("none")).toBe("None");
  });
});
