export type CameraState =
  | "stopped"
  | "starting"
  | "running"
  | "requestingPermission"
  | "enumerating"
  | "permissionRequired"
  | "unavailable"
  | "error";

export type HandTrackingState = "waiting" | "modelLoading" | "modelReady" | "detected" | "lost" | "error";
export type ControlMode = "disabled" | "cursor" | "window" | "airKeyboard";
export type ActiveGesture = "none" | "cursor" | "click" | "drag" | "scroll" | "rightClick";

export interface RuntimeSnapshot {
  camera: CameraState;
  handTracking: HandTrackingState;
  controlMode: ControlMode;
  activeGesture: ActiveGesture;
  framesPerSecond: number | null;
  accessibilityGranted: boolean;
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
    detected: "Hand detected",
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
