import { AlertTriangle, Check, CircleHelp, Hand, Play, RefreshCw, RotateCcw, ShieldCheck, Square, Video } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CameraDevice, CameraFrame, CameraStatusEvent } from "./domain/camera";
import { DEFAULT_GESTURE_STUDIO, gestureConflicts, updateGestureBinding, updateProfileGestureSettings, type GestureAction, type GestureBinding, type GestureId } from "./domain/gestureStudio";
import type { UserPreferences } from "./domain/userPreferences";
import type { RuntimeSnapshot } from "./domain/runtimeStatus";
import { gestureLabel, trackingLabel } from "./domain/runtimeStatus";

const HAND_CONNECTIONS: [number, number][] = [
  [0, 1], [1, 2], [2, 3], [3, 4], [0, 5], [5, 6], [6, 7], [7, 8],
  [0, 9], [9, 10], [10, 11], [11, 12], [0, 13], [13, 14], [14, 15], [15, 16],
  [0, 17], [17, 18], [18, 19], [19, 20], [5, 9], [9, 13], [13, 17],
];

const GESTURES: { id: GestureId; name: string; description: string; actions: GestureAction[] }[] = [
  { id: "indexPinch", name: "Index pinch", description: "Bring thumb and index fingertip together.", actions: ["clickAndDrag", "click", "drag", "disabled"] },
  { id: "middlePinch", name: "Middle-finger pinch", description: "Bring thumb and middle fingertip together.", actions: ["rightClick", "disabled"] },
  { id: "twoFingerScroll", name: "Two-finger scroll", description: "Move the extended index and middle fingers vertically.", actions: ["scroll", "disabled"] },
  { id: "threeFingerSwipe", name: "Three-finger swipe", description: "Swipe horizontally with three fingers extended.", actions: ["switchApplication", "disabled"] },
];

const ACTION_LABELS: Record<GestureAction, string> = {
  clickAndDrag: "Click and drag", click: "Click", drag: "Drag", rightClick: "Right click",
  scroll: "Scroll", switchApplication: "Switch application", disabled: "Disabled",
};

function HandOverlay({ frame }: { frame: CameraFrame }) {
  return <svg className="camera-overlay" viewBox={`0 0 ${frame.width} ${frame.height}`} preserveAspectRatio="xMidYMid meet" role="img" aria-label="Live hand joint landmarks">
    <g transform={`translate(${frame.width} 0) scale(-1 1)`}>{frame.palms.map((palm, palmIndex) => <g key={palmIndex}>
      {palm.fingerLandmarks && <g className="finger-landmarks">{HAND_CONNECTIONS.map(([from, to]) => <line key={`${from}-${to}`} x1={palm.fingerLandmarks?.[from][0]} y1={palm.fingerLandmarks?.[from][1]} x2={palm.fingerLandmarks?.[to][0]} y2={palm.fingerLandmarks?.[to][1]} />)}{palm.fingerLandmarks.map(([x, y], index) => <circle key={index} cx={x} cy={y} r="3" />)}</g>}
      <text x={palm.bounds[0] + 4} y={Math.max(12, palm.bounds[1] - 5)}>{Math.round(palm.confidence * 100)}%</text>
    </g>)}</g>
  </svg>;
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="gesture-metric"><span>{label}</span><strong>{value}</strong></div>;
}

export function GestureStudioSection({
  connection, preferences, status, error, savePreferences, listCameras, startCamera, stopCamera, setGestureTestMode,
}: {
  connection: "preview" | "connected" | "error";
  preferences: UserPreferences;
  status: RuntimeSnapshot | null;
  error: string | null;
  savePreferences: (preferences: UserPreferences) => Promise<boolean>;
  listCameras: () => Promise<CameraDevice[] | null>;
  startCamera: (deviceId: string) => Promise<boolean>;
  stopCamera: () => Promise<boolean>;
  setGestureTestMode: (enabled: boolean) => Promise<boolean>;
}) {
  const [draft, setDraft] = useState(preferences);
  const [frame, setFrame] = useState<CameraFrame | null>(null);
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [selectedCamera, setSelectedCamera] = useState("");
  const [selectedGesture, setSelectedGesture] = useState<GestureId>("indexPinch");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [transitions, setTransitions] = useState<{ id: number; text: string }[]>([]);
  const previousStatus = useRef<{ gesture?: string; tracking?: string }>({});
  const settings = draft.gestureStudio ?? DEFAULT_GESTURE_STUDIO;
  const conflicts = useMemo(() => gestureConflicts(settings), [settings]);
  const connected = connection === "connected";
  const cameraRunning = status?.camera === "running";
  const testing = status?.controlMode === "gestureTest";
  const canStartTest = connected && cameraRunning && status?.handTracking === "detected" && status.controlMode === "disabled";
  const selectedBinding = settings.bindings.find((binding) => binding.gesture === selectedGesture) ?? DEFAULT_GESTURE_STUDIO.bindings[0];
  const cooldownApplicable = selectedGesture !== "twoFingerScroll";
  const diagnostics = status?.diagnostics;

  useEffect(() => setDraft(preferences), [preferences]);
  useEffect(() => {
    const nextGesture = status?.activeGesture;
    const nextTracking = status?.handTracking;
    const entries: string[] = [];
    if (nextGesture && previousStatus.current.gesture && nextGesture !== previousStatus.current.gesture) {
      entries.push(`${gestureLabel(previousStatus.current.gesture as RuntimeSnapshot["activeGesture"])} → ${gestureLabel(nextGesture)}`);
    }
    if (nextTracking === "lost" && previousStatus.current.tracking !== "lost") entries.push("Tracking lost · gesture state reset");
    if (entries.length) setTransitions((current) => [...entries.map((text, index) => ({ id: Date.now() + index, text })), ...current].slice(0, 5));
    previousStatus.current = { gesture: nextGesture, tracking: nextTracking };
  }, [status?.activeGesture, status?.handTracking]);
  useEffect(() => {
    if (!connected) return;
    let disposed = false;
    let stopFrame: UnlistenFn | undefined;
    let stopCameraStatus: UnlistenFn | undefined;
    void listen<CameraFrame>("camera-frame", (event) => setFrame(event.payload)).then((stop) => disposed ? stop() : stopFrame = stop);
    void listen<CameraStatusEvent>("camera-status", (event) => {
      setMessage(event.payload.message ?? "");
      if (event.payload.state === "stopped" || event.payload.state === "error") setFrame(null);
    }).then((stop) => disposed ? stop() : stopCameraStatus = stop);
    return () => { disposed = true; stopFrame?.(); stopCameraStatus?.(); };
  }, [connected]);

  async function findCameras() {
    setBusy(true);
    const found = await listCameras();
    setBusy(false);
    if (found === null) { setMessage(error ?? "Camera permission is unavailable."); return; }
    setCameras(found);
    setSelectedCamera(found[0]?.id ?? "");
    setMessage(found.length ? `${found.length} camera${found.length === 1 ? "" : "s"} available.` : "No camera was found.");
  }

  async function toggleCamera() {
    setBusy(true);
    const success = cameraRunning ? await stopCamera() : selectedCamera ? await startCamera(selectedCamera) : false;
    setBusy(false);
    setMessage(success ? cameraRunning ? "Camera stopped." : "Camera started. Waiting for hand tracking." : error ?? "Camera action could not be completed.");
  }

  async function toggleTest() {
    setBusy(true);
    const success = await setGestureTestMode(!testing);
    setBusy(false);
    setMessage(success ? testing ? "Safe test stopped and gesture state reset." : "Safe test started. Native input is blocked in Rust." : error ?? "Safe test could not be started.");
  }

  async function persist(next: UserPreferences, savedMessage: string) {
    setBusy(true);
    const saved = await savePreferences(next);
    setBusy(false);
    setMessage(saved ? savedMessage : error ?? "Settings were not saved.");
    if (saved) setDraft(next);
  }

  function changeBinding(patch: Partial<GestureBinding>) {
    const nextSettings = updateGestureBinding(settings, selectedGesture, patch);
    setDraft((current) => updateProfileGestureSettings(current, nextSettings));
  }

  async function saveSettings() {
    if (conflicts.length) { setMessage("Resolve gesture conflicts before saving."); return; }
    await persist(draft, "Gesture settings saved to this profile.");
  }

  async function resetSettings() {
    const next = updateProfileGestureSettings(draft, DEFAULT_GESTURE_STUDIO);
    await persist(next, "This profile’s gesture settings were restored.");
  }

  return <section className="gesture-studio" aria-label="Gesture Studio workspace">
    <div className="gesture-studio-toolbar">
      <label>Profile<select aria-label="Gesture profile" value={draft.activeProfileId} disabled={!connected || busy} onChange={(event) => {
        const profile = draft.profiles.find((item) => item.id === event.currentTarget.value);
        if (profile) void persist({ ...draft, ...profile.settings, activeProfileId: profile.id }, `${profile.name} profile loaded.`);
      }}>{draft.profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}</select></label>
      <div className="gesture-studio-actions"><button type="button" className="secondary-button" onClick={() => void resetSettings()} disabled={!connected || busy}><RotateCcw size={14} />Reset profile</button><button type="button" className="primary-button" onClick={() => void saveSettings()} disabled={!connected || busy || conflicts.length > 0}><Check size={14} />Save changes</button></div>
    </div>
    <p className="gesture-save-note">Saved bindings apply the next time Cursor or Window control starts. Safe Testing Mode reads saved changes immediately.</p>
    <div className="gesture-studio-layout">
      <div className="gesture-studio-preview-column">
        <div className={`camera-stage gesture-studio-stage ${frame ? "has-frame" : ""}`}>
          {frame ? <img className="camera-frame" src={`data:image/jpeg;base64,${frame.jpegBase64}`} alt="Live camera preview" /> : <div className="camera-empty"><span><Video size={25} aria-hidden="true" /></span><strong>{cameraRunning ? "Waiting for the first frame" : "Camera preview is off"}</strong><small>Frames and landmarks are processed locally.</small></div>}
          {frame && <HandOverlay frame={frame} />}
          <span className={`camera-live-badge ${cameraRunning ? "active" : ""}`}><span />{cameraRunning ? "LIVE" : "OFF"}</span>
          {testing && <span className="gesture-test-badge"><ShieldCheck size={12} /> SAFE TEST · INPUT BLOCKED</span>}
        </div>
        <div className="gesture-tracking-line" aria-live="off"><span><Hand size={14} />{trackingLabel(status?.handTracking ?? "waiting")}</span><strong>{status?.gestureConfidence == null ? "Confidence —" : `Confidence ${Math.round(status.gestureConfidence * 100)}%`}</strong></div>
        <div className="gesture-studio-camera-actions">
          <select aria-label="Camera device" value={selectedCamera} disabled={!connected || !cameras.length || cameraRunning || busy} onChange={(event) => setSelectedCamera(event.currentTarget.value)}><option value="">Select a camera</option>{cameras.map((camera) => <option key={camera.id} value={camera.id}>{camera.name}</option>)}</select>
          <button type="button" className="secondary-button" disabled={!connected || cameraRunning || busy} onClick={() => void findCameras()}><RefreshCw size={14} />Find camera</button>
          <button type="button" className="secondary-button" disabled={!connected || busy || (!cameraRunning && !selectedCamera)} onClick={() => void toggleCamera()}>{cameraRunning ? <><Square size={13} />Stop</> : <><Play size={14} />Start</>}</button>
        </div>
        <div className="gesture-test-control">
          <div><strong>Safe Testing Mode</strong><small>Preview recognition and predicted actions without sending Windows input.</small></div>
          <button type="button" className={testing ? "danger-button" : "primary-button"} disabled={busy || (!testing && !canStartTest)} onClick={() => void toggleTest()}>{testing ? <><Square size={13} />Stop test</> : <><Play size={14} />Start test</>}</button>
        </div>
        {!connected && <div className="camera-runtime-note" role="status"><CircleHelp size={15} />Open the Windows desktop app to access camera and profile services.</div>}
        {connected && !cameraRunning && <div className="camera-runtime-note" role="status"><AlertTriangle size={15} />Start the camera before beginning a safe gesture test.</div>}
      </div>
      <div className="gesture-studio-controls">
        <div className="gesture-section-heading"><div><span className="eyebrow">GESTURE MAP</span><h2>Gestures</h2></div><span className="gesture-count">{GESTURES.length} triggers</span></div>
        <div className="gesture-choice-list" aria-label="Available gestures">{GESTURES.map((gesture) => {
          const binding = settings.bindings.find((item) => item.gesture === gesture.id);
          return <button type="button" key={gesture.id} className={`gesture-choice ${selectedGesture === gesture.id ? "selected" : ""}`} aria-pressed={selectedGesture === gesture.id} onClick={() => setSelectedGesture(gesture.id)}>
            <span className="gesture-choice-icon"><Hand size={17} /></span><span className="gesture-choice-copy"><strong>{gesture.name}</strong><small>{gesture.description}</small></span><span className="gesture-choice-action">{ACTION_LABELS[binding?.action ?? "disabled"]}</span>
          </button>;
        })}</div>
        <div className="gesture-binding-panel">
          <div className="gesture-section-heading"><div><span className="eyebrow">SELECTED GESTURE</span><h2>{GESTURES.find((gesture) => gesture.id === selectedGesture)?.name}</h2></div><span className="gesture-binding-current">{ACTION_LABELS[selectedBinding.action]}</span></div>
          <label className="gesture-field">Assigned action<select aria-label="Assigned action" value={selectedBinding.action} disabled={!connected || busy} onChange={(event) => changeBinding({ action: event.currentTarget.value as GestureAction })}>{GESTURES.find((gesture) => gesture.id === selectedGesture)?.actions.map((action) => <option key={action} value={action}>{ACTION_LABELS[action]}</option>)}</select></label>
          <label className="gesture-field">Sensitivity <output>{selectedBinding.sensitivity.toFixed(2)}×</output><input aria-label="Gesture sensitivity" type="range" min="0.6" max="1.4" step="0.05" value={selectedBinding.sensitivity} disabled={!connected || busy} onChange={(event) => changeBinding({ sensitivity: Number(event.currentTarget.value) })} /></label>
          <label className="gesture-field">Cooldown <output>{cooldownApplicable ? `${selectedBinding.cooldownMs} ms` : "Continuous gesture"}</output><input aria-label="Gesture cooldown" type="range" min="100" max="2000" step="50" value={selectedBinding.cooldownMs} disabled={!connected || busy || !cooldownApplicable} onChange={(event) => changeBinding({ cooldownMs: Number(event.currentTarget.value) })} />{!cooldownApplicable && <small>Continuous scrolling is not cooldown-limited.</small>}</label>
          {conflicts.length > 0 && <div className="gesture-conflicts" role="alert"><AlertTriangle size={15} /><div><strong>Gesture conflicts</strong><ul>{conflicts.map((conflict) => <li key={conflict}>{conflict}</li>)}</ul></div></div>}
          <div className="gesture-prediction" aria-live="off"><span>Recognized state</span><strong>{gestureLabel(status?.activeGesture ?? "none")}</strong><span>Predicted action</span><strong>{status?.gestureTestAction ? ACTION_LABELS[status.gestureTestAction] : testing ? "No action yet" : "Testing stopped"}</strong></div>
          <div className="gesture-transition-panel"><strong>Recent transitions</strong>{transitions.length ? <ol>{transitions.map((transition) => <li key={transition.id}>{transition.text}</li>)}</ol> : <span>Transitions will appear during a safe test.</span>}</div>
        </div>
      </div>
    </div>
    <section className="gesture-diagnostics" aria-label="Performance diagnostics"><div className="gesture-section-heading"><div><span className="eyebrow">LIVE DIAGNOSTICS</span><h2>Performance</h2></div></div><div className="gesture-metrics-grid">
      <Metric label="Camera FPS" value={diagnostics?.cameraFps == null ? "—" : diagnostics.cameraFps.toFixed(1)} />
      <Metric label="Tracking FPS" value={diagnostics?.trackingFps == null ? "—" : diagnostics.trackingFps.toFixed(1)} />
      <Metric label="Inference" value={diagnostics?.inferenceLatencyMs == null ? "—" : `${diagnostics.inferenceLatencyMs.toFixed(1)} ms`} />
      <Metric label="Gesture pipeline" value={diagnostics?.gestureLatencyMs == null ? "—" : `${diagnostics.gestureLatencyMs.toFixed(2)} ms`} />
      <Metric label="Tracking losses" value={diagnostics ? String(diagnostics.trackingLossCount) : "—"} />
      <Metric label="Input events" value={diagnostics ? String(diagnostics.inputEventCount) : "—"} />
      <Metric label="CPU" value={diagnostics?.processCpuPercent == null ? "Unavailable" : `${diagnostics.processCpuPercent.toFixed(1)}%`} />
      <Metric label="Memory" value={diagnostics?.processMemoryBytes == null ? "Unavailable" : `${(diagnostics.processMemoryBytes / 1024 / 1024).toFixed(0)} MB`} />
    </div></section>
    {message && <p className="gesture-studio-message" role="status">{message}</p>}
  </section>;
}
