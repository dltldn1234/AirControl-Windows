export interface CameraDevice {
  id: string;
  name: string;
}

export interface CameraFrame {
  jpegBase64: string;
  width: number;
  height: number;
}

export interface CameraStatusEvent {
  state: import("./runtimeStatus").CameraState;
  message: string | null;
}
