export interface CameraDevice {
  id: string;
  name: string;
}

export interface CameraFrame {
  jpegBase64: string;
  width: number;
  height: number;
  palms: PalmDetection[];
}

export interface PalmDetection {
  bounds: [number, number, number, number];
  landmarks: [number, number][];
  fingerLandmarks: [number, number][] | null;
  confidence: number;
}

export interface CameraStatusEvent {
  state: import("./runtimeStatus").CameraState;
  message: string | null;
}

export interface HandTrackingStatusEvent {
  state: import("./runtimeStatus").HandTrackingState;
  message: string | null;
}
