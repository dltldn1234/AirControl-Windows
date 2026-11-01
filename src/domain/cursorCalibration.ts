export interface CursorCalibrationBounds {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export type CalibrationEdge = keyof CursorCalibrationBounds;

export const DEFAULT_CURSOR_CALIBRATION: CursorCalibrationBounds = {
  left: 0,
  top: 0,
  right: 1,
  bottom: 1,
};

export function moveCalibrationEdge(
  bounds: CursorCalibrationBounds,
  edge: CalibrationEdge,
  requestedValue: number,
): CursorCalibrationBounds {
  const safeValue = Number.isFinite(requestedValue)
    ? Math.min(1, Math.max(0, requestedValue))
    : DEFAULT_CURSOR_CALIBRATION[edge];
  const next = { ...bounds };
  const minimumSpan = 0.15;
  if (edge === "left") next.left = Math.min(safeValue, next.right - minimumSpan);
  if (edge === "right") next.right = Math.max(safeValue, next.left + minimumSpan);
  if (edge === "top") next.top = Math.min(safeValue, next.bottom - minimumSpan);
  if (edge === "bottom") next.bottom = Math.max(safeValue, next.top + minimumSpan);
  return next;
}
