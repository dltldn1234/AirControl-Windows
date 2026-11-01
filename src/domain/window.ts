export interface WindowRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface WindowInfo {
  handle: string;
  title: string;
  processId: number;
  rect: WindowRect;
  minimized: boolean;
  maximized: boolean;
}
