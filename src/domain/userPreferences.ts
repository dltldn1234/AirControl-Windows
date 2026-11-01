export interface UserPreferences {
  cursorSensitivity: number;
  cursorSmoothing: number;
  pinchSensitivity: number;
  windowSensitivity: number;
  scrollSensitivity: number;
  reverseScrollDirection: boolean;
  rightClickEnabled: boolean;
  cameraMirrored: boolean;
  appSwitchEnabled: boolean;
  cursorCalibration: { left: number; top: number; right: number; bottom: number };
}
