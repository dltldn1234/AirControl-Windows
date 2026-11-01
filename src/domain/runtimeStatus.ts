export type CameraState =
  | "stopped"
  | "starting"
  | "running"
  | "requestingPermission"
  | "enumerating"
  | "permissionRequired"
  | "unavailable"
  | "error";

export type HandTrackingState = "waiting" | "modelLoading" | "modelReady" | "palmDetected" | "detected" | "lost" | "error";
export type ControlMode = "disabled" | "cursor" | "window" | "airKeyboard";
export type ActiveGesture = "none" | "cursor" | "click" | "drag" | "scroll" | "rightClick";

export interface RuntimeSnapshot {
  camera: CameraState;
  handTracking: HandTrackingState;
  controlMode: ControlMode;
  activeGesture: ActiveGesture;
  framesPerSecond: number | null;
}

export function canEnableCursorControl(
  status: RuntimeSnapshot | null,
  connection: "preview" | "connected" | "error",
): boolean {
  return connection === "connected"
    && status?.camera === "running"
    && status.handTracking === "detected"
    && status.controlMode === "disabled";
}

export function cameraLabel(state: CameraState | "unavailable"): string {
  const labels: Record<CameraState | "unavailable", string> = {
    stopped: "Stopped",
    starting: "Starting",
    running: "Connected",
    requestingPermission: "Permission needed",
    enumerating: "Searching",
    permissionRequired: "Permission needed",
    unavailable: "Not connected",
    error: "Error",
  };
  return labels[state];
}

export function trackingLabel(state: HandTrackingState): string {
  const labels: Record<HandTrackingState, string> = {
    waiting: "Waiting",
    modelLoading: "Loading model",
    modelReady: "Model loaded",
    palmDetected: "Palm detected",
    detected: "21 landmarks",
    lost: "Tracking lost",
    error: "Model error",
  };
  return labels[state];
}

export function gestureLabel(gesture: ActiveGesture): string {
  if (gesture === "none") return "None";
  if (gesture === "rightClick") return "Right click";
  return gesture.charAt(0).toUpperCase() + gesture.slice(1);
}
