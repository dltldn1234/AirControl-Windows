import {
  Activity,
  AlertTriangle,
  AppWindow,
  Camera,
  Check,
  ChevronRight,
  CircleHelp,
  Command,
  Focus,
  Hand,
  Keyboard,
  MousePointer2,
  Settings2,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Play,
  RefreshCw,
  Square,
  Video,
  WandSparkles,
} from "lucide-react";
import { useEffect, useReducer, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useRuntimeStore } from "./state/runtimeStore";
import type { RuntimeSnapshot } from "./domain/runtimeStatus";
import { DEFAULT_CURSOR_CALIBRATION, moveCalibrationEdge } from "./domain/cursorCalibration";
import type { CameraDevice, CameraFrame, CameraStatusEvent, HandTrackingStatusEvent } from "./domain/camera";
import { createControlProfile, deleteControlProfile, normalizeExecutableIdentity, removeAppProfileRule, renameControlProfile, restoreControlProfile, upsertAppProfileRule, withProfileSettings, type ControlProfile, type UserPreferences } from "./domain/userPreferences";
import type { WindowInfo } from "./domain/window";
import { cameraLabel, canEnableCursorControl, gestureLabel, trackingLabel } from "./domain/runtimeStatus";
import { commandKey, KEYBOARD_ROWS, type KeyboardKey } from "./keyboardLayout";
import { INITIAL_KEYBOARD_FEEDBACK, reduceKeyboardFeedback } from "./keyboardFeedback";
import { CommandPaletteSection } from "./CommandPaletteSection";

const HAND_CONNECTIONS: [number, number][] = [
  [0, 1], [1, 2], [2, 3], [3, 4],
  [0, 5], [5, 6], [6, 7], [7, 8],
  [0, 9], [9, 10], [10, 11], [11, 12],
  [0, 13], [13, 14], [14, 15], [15, 16],
  [0, 17], [17, 18], [18, 19], [19, 20],
  [5, 9], [9, 13], [13, 17],
];

const navigation = [
  { label: "Overview", icon: Activity, group: "Workspace" },
  { label: "Camera & tracking", icon: Camera, group: "Workspace" },
  { label: "Gestures", icon: Hand, group: "Workspace" },
  { label: "Command palette", icon: Command, group: "Workspace" },
  { label: "Window control", icon: AppWindow, group: "Workspace" },
  { label: "Air Keyboard", icon: Keyboard, group: "Workspace" },
  { label: "Calibration", icon: Focus, group: "Workspace" },
  { label: "Performance", icon: SlidersHorizontal, group: "Manage" },
  { label: "Settings", icon: Settings2, group: "Manage" },
];

const sectionDescriptions: Record<string, string> = {
  Overview: "Your gesture controls, at a glance.",
  "Camera & tracking": "Camera capture and hand landmarks.",
  Gestures: "Choose how your gestures map to actions.",
  "Command palette": "Run app, window, and media actions with a hand gesture.",
  "Window control": "Move, resize, and switch between apps.",
  "Air Keyboard": "Type with a floating, gesture-driven keyboard.",
  Calibration: "Tune tracking to your reach and camera position.",
  Performance: "Live frame rate and inference timing.",
  Settings: "Preferences, permissions, and privacy.",
};

function StatusCard({
  icon: Icon,
  label,
  value,
  detail,
  tone = "neutral",
}: {
  icon: typeof Camera;
  label: string;
  value: string;
  detail: string;
  tone?: "neutral" | "good";
}) {
  return (
    <article className="status-card">
      <div className="status-card-top">
        <span className="status-icon"><Icon size={16} aria-hidden="true" /></span>
        <span className="eyebrow">{label}</span>
        {tone === "good" && <span className="status-check"><Check size={13} aria-label="Ready" /></span>}
      </div>
      <strong>{value}</strong>
      <span className="status-detail">{detail}</span>
    </article>
  );
}

function RuntimeUnavailable({ detail = "Live camera and system status will appear here when AirControl runs as a Tauri desktop app." }: { detail?: string }) {
  return (
    <div className="runtime-notice" role="status">
      <span className="notice-icon"><CircleHelp size={17} aria-hidden="true" /></span>
      <div>
        <strong>Desktop runtime is not connected</strong>
        <p>{detail}</p>
      </div>
    </div>
  );
}

function Overview({
  status,
  connection,
  error,
  setCursorControl,
}: {
  status: RuntimeSnapshot | null;
  connection: "preview" | "connected" | "error";
  error: string | null;
  setCursorControl: (enabled: boolean) => Promise<boolean>;
}) {
  const camera = status?.camera ?? "unavailable";
  const tracking = status?.handTracking ?? "waiting";
  const mode = status?.controlMode ?? "disabled";
  const modeLabel = mode === "commandPalette" ? "Command palette" : mode === "airKeyboard" ? "Air Keyboard" : mode;
  const gesture = status?.activeGesture ?? "none";
  const fps = status?.framesPerSecond;
  const isEnabled = mode === "cursor";
  const canEnable = canEnableCursorControl(status, connection);

  return (
    <>
      <section className="welcome-panel">
        <div className="welcome-copy">
          <div className="welcome-kicker"><Sparkles size={14} aria-hidden="true" /> YOUR DESKTOP, HANDS-FREE</div>
          <h2>Control your PC,<br /><span>one gesture at a time.</span></h2>
          <p>AirControl is being set up. Your camera and system controls stay off until you choose to enable them.</p>
          <div className="safety-note"><ShieldCheck size={15} aria-hidden="true" /> System input is disabled by default</div>
        </div>
        <div className="gesture-art" aria-hidden="true">
          <div className="orbit orbit-one" />
          <div className="orbit orbit-two" />
          <div className="gesture-hand"><Hand size={88} strokeWidth={1.1} /></div>
          <span className="orbit-dot dot-one" /><span className="orbit-dot dot-two" />
          <span className="orbit-dot dot-three" />
        </div>
        <div className="welcome-index">01 <span>/ 04</span></div>
      </section>

      <div className="section-heading status-heading">
        <div><h2>System status</h2><p>Current state from the local AirControl runtime.</p></div>
        <span className="live-label"><span className="live-dot" /> {status ? "RUNTIME STATUS" : "PREVIEW"}</span>
      </div>

      {!status && <RuntimeUnavailable />}

      <section className="status-grid" aria-label="System status">
        <StatusCard icon={Camera} label="CAMERA" value={cameraLabel(camera)} detail={camera === "running" ? "Capture is active" : "Waiting for camera service"} />
        <StatusCard icon={Hand} label="HAND TRACKING" value={trackingLabel(tracking)} detail={tracking === "detected" ? "Palm and all 21 joints are tracked" : tracking === "palmDetected" ? "Palm found; waiting for confident joints" : "No hand landmarks received"} />
        <StatusCard icon={MousePointer2} label="CONTROL MODE" value={mode === "disabled" ? "Off" : modeLabel} detail={mode === "disabled" ? "No system input is being sent" : "Explicitly enabled"} />
        <StatusCard icon={WandSparkles} label="GESTURE" value={gestureLabel(gesture)} detail={fps == null ? "FPS unavailable" : `${fps.toFixed(0)} frames per second`} />
      </section>

      <section className="control-panel" aria-label="Cursor control">
        <div><h2>Cursor control</h2><p>{isEnabled ? "Hand landmarks currently control the Windows pointer." : "Control starts only after a camera and confident hand pose are available."}</p></div>
        <button type="button" className={isEnabled ? "secondary-button" : "primary-button"} disabled={!isEnabled && !canEnable} onClick={() => void setCursorControl(!isEnabled)}>
          {isEnabled ? "Stop cursor control" : "Start cursor control"}
        </button>
        {error && <p className="camera-message error" role="alert">{error}</p>}
      </section>

      <div className="section-heading next-heading">
        <div><h2>Getting started</h2><p>AirControl will guide you through setup as each capability becomes available.</p></div>
      </div>
      <section className="setup-list" aria-label="Setup progress">
        <div className="setup-row complete"><span className="step-number"><Check size={14} /></span><span className="setup-text"><strong>AirControl installed</strong><small>Windows desktop shell is ready</small></span><span className="step-state">READY</span></div>
        <div className="setup-row"><span className="step-number">02</span><span className="setup-text"><strong>Connect a camera</strong><small>Camera capture is the next development phase</small></span><span className="step-state pending">UP NEXT</span></div>
        <div className="setup-row"><span className="step-number">03</span><span className="setup-text"><strong>Calibrate your gestures</strong><small>Personalize movement and pinch sensitivity</small></span><span className="step-state pending">COMING SOON</span></div>
      </section>
    </>
  );
}

function PlannedSection({ title }: { title: string }) {
  return (
    <section className="planned-panel">
      <span className="planned-icon"><WandSparkles size={21} aria-hidden="true" /></span>
      <span className="eyebrow">IN DEVELOPMENT</span>
      <h2>{title}</h2>
      <p>{sectionDescriptions[title]} This section will become interactive when its service is implemented. No simulated controls are shown.</p>
      <div className="planned-status"><span className="status-dot" /> Not available yet</div>
    </section>
  );
}

function CameraSection({
  connection,
  cameraState,
  handTrackingState,
  error,
  listCameras,
  startCamera,
  stopCamera,
  setCameraState,
  setHandTrackingState,
}: {
  connection: "preview" | "connected" | "error";
  cameraState: RuntimeSnapshot["camera"] | "unavailable";
  handTrackingState: RuntimeSnapshot["handTracking"];
  error: string | null;
  listCameras: () => Promise<CameraDevice[] | null>;
  startCamera: (deviceId: string) => Promise<boolean>;
  stopCamera: () => Promise<boolean>;
  setCameraState: (state: RuntimeSnapshot["camera"]) => void;
  setHandTrackingState: (state: RuntimeSnapshot["handTracking"]) => void;
}) {
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [selectedCamera, setSelectedCamera] = useState("");
  const [frame, setFrame] = useState<CameraFrame | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const canUseCamera = connection === "connected";

  useEffect(() => {
    if (!canUseCamera) return;
    let disposed = false;
    let unlistenFrame: UnlistenFn | undefined;
    let unlistenStatus: UnlistenFn | undefined;
    let unlistenTracking: UnlistenFn | undefined;
    void listen<CameraFrame>("camera-frame", (event) => {
      setFrame(event.payload);
    }).then((unlisten) => {
      if (disposed) unlisten();
      else unlistenFrame = unlisten;
    });
    void listen<CameraStatusEvent>("camera-status", (event) => {
      setCameraState(event.payload.state);
      setMessage(event.payload.message ?? "");
      if (event.payload.state === "stopped" || event.payload.state === "error") setFrame(null);
    }).then((unlisten) => {
      if (disposed) unlisten();
      else unlistenStatus = unlisten;
    });
    void listen<HandTrackingStatusEvent>("hand-tracking-status", (event) => {
      setHandTrackingState(event.payload.state);
      if (event.payload.message) setMessage(event.payload.message);
    }).then((unlisten) => {
      if (disposed) unlisten();
      else unlistenTracking = unlisten;
    });
    return () => {
      disposed = true;
      unlistenFrame?.();
      unlistenStatus?.();
      unlistenTracking?.();
    };
  }, [canUseCamera, setCameraState, setHandTrackingState]);

  async function refreshCameras() {
    setLoading(true);
    setMessage("Requesting camera access and searching for devices…");
    const devices = await listCameras();
    setLoading(false);
    if (devices === null) {
      setMessage(error ?? "Camera permission is unavailable. Check Windows camera privacy settings and try again.");
      return;
    }
    setCameras(devices);
    setSelectedCamera(devices[0]?.id ?? "");
    setMessage(devices.length === 0 ? "No cameras were found. Connect a camera and search again." : `${devices.length} camera${devices.length === 1 ? "" : "s"} found. Camera frames stay on this PC.`);
  }

  async function toggleCapture() {
    if (cameraState === "running" || cameraState === "starting") {
      await stopCamera();
      return;
    }
    if (!selectedCamera) return;
    setMessage("Starting camera preview…");
    if (!(await startCamera(selectedCamera))) setMessage(error ?? "Camera could not be started.");
  }

  return (
    <section className="camera-panel">
      <div className="camera-preview-column">
        <div className={`camera-stage ${frame ? "has-frame" : ""}`}>
          {frame ? <img className="camera-frame" src={`data:image/jpeg;base64,${frame.jpegBase64}`} alt="Live camera preview" /> : <div className="camera-empty"><span><Video size={25} aria-hidden="true" /></span><strong>{cameraState === "running" ? "Waiting for the first frame" : "Camera preview is off"}</strong><small>Frames are processed and displayed locally.</small></div>}
          {frame && frame.palms.length > 0 && <svg className="camera-overlay" viewBox={`0 0 ${frame.width} ${frame.height}`} preserveAspectRatio="xMidYMid meet" role="img" aria-label="Detected hand regions and finger-joint landmarks"><g transform={`translate(${frame.width} 0) scale(-1 1)`}>{frame.palms.map((palm, palmIndex) => <g key={palmIndex}><rect x={palm.bounds[0]} y={palm.bounds[1]} width={palm.bounds[2] - palm.bounds[0]} height={palm.bounds[3] - palm.bounds[1]} rx="3" /><text x={palm.bounds[0] + 4} y={Math.max(12, palm.bounds[1] - 5)}>{Math.round(palm.confidence * 100)}%</text>{palm.fingerLandmarks ? <g className="finger-landmarks">{HAND_CONNECTIONS.map(([from, to]) => <line key={`${from}-${to}`} x1={palm.fingerLandmarks?.[from][0]} y1={palm.fingerLandmarks?.[from][1]} x2={palm.fingerLandmarks?.[to][0]} y2={palm.fingerLandmarks?.[to][1]} />)}{palm.fingerLandmarks.map(([x, y], index) => <circle key={index} cx={x} cy={y} r="3" />)}</g> : palm.landmarks.map(([x, y], index) => <circle key={index} cx={x} cy={y} r="4" />)}</g>)}</g></svg>}
          <span className={`camera-live-badge ${cameraState === "running" ? "active" : ""}`}><span />{cameraState === "running" ? "LIVE" : cameraState === "starting" ? "STARTING" : "OFF"}</span>
          {frame && <span className="camera-resolution">{frame.width} × {frame.height}</span>}
        </div>
        <p className="camera-privacy"><ShieldCheck size={14} aria-hidden="true" /> Preview frames are not saved or sent to a server.</p>
      </div>
      <div className="camera-controls">
        <div className="camera-controls-heading"><div><h2>Camera source</h2><p>Choose a local camera to see a live preview.</p></div><span className={`camera-state-pill ${cameraState}`}>{cameraLabel(cameraState)}</span></div>
        <div className="camera-tracking-status" role="status"><strong>Hand tracking</strong><span>{trackingLabel(handTrackingState)}</span><small>{handTrackingState === "modelReady" ? "Models loaded locally; waiting for the first camera frame." : handTrackingState === "detected" ? frame?.palms.some((palm) => palm.fingerLandmarks?.length === 21) ? "Palm and all 21 finger-joint landmarks are visible." : "Palm found; waiting for a confident 21-point pose." : handTrackingState === "lost" ? "No palm is currently visible; the detector keeps searching." : handTrackingState === "error" ? "Hand inference failed. Check the runtime error and restart camera capture." : "Local hand inference is waiting for a camera frame."}</small></div>
        <label className="camera-select-label" htmlFor="camera-device">Available cameras</label>
        <select id="camera-device" value={selectedCamera} disabled={!canUseCamera || cameras.length === 0 || cameraState === "running"} onChange={(event) => setSelectedCamera(event.currentTarget.value)}>
          {cameras.length === 0 ? <option value="">Search for cameras first</option> : cameras.map((camera) => <option key={camera.id} value={camera.id}>{camera.name}</option>)}
        </select>
        <div className="camera-actions">
          <button type="button" className="secondary-button" onClick={() => void refreshCameras()} disabled={!canUseCamera || loading || cameraState === "running"}><RefreshCw size={14} className={loading ? "spinning" : ""} />{loading ? "Searching…" : "Find cameras"}</button>
          <button type="button" className="primary-button camera-toggle" onClick={() => void toggleCapture()} disabled={!canUseCamera || (cameraState !== "running" && cameraState !== "starting" && !selectedCamera)}>{cameraState === "running" || cameraState === "starting" ? <><Square size={13} /> Stop preview</> : <><Play size={14} /> Start preview</>}</button>
        </div>
        {!canUseCamera && <div className="camera-runtime-note" role="status"><AlertTriangle size={15} /> Open AirControl as a desktop app to access its camera service.</div>}
        {message && <p className={`camera-message ${message.toLowerCase().includes("denied") || message.toLowerCase().includes("could not") ? "error" : ""}`} role="status">{message}</p>}
        <div className="camera-permission-note"><strong>Privacy</strong><span>Camera access is requested only when you search for devices. No frames leave this computer.</span></div>
      </div>
    </section>
  );
}

function SettingsSection({
  connection,
  initialPreferences,
  savePreferences,
}: {
  connection: "preview" | "connected" | "error";
  initialPreferences: UserPreferences;
  savePreferences: (preferences: UserPreferences) => Promise<boolean>;
}) {
  const [preferences, setPreferences] = useState(initialPreferences);
  const [saveState, setSaveState] = useState("Changes apply only after saving.");
  const [newProfileName, setNewProfileName] = useState("");
  const [appExecutable, setAppExecutable] = useState("");
  const [appProfileId, setAppProfileId] = useState(initialPreferences.activeProfileId);
  const [renameValue, setRenameValue] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const runtimeAvailable = connection === "connected";
  const canSave = runtimeAvailable && !isSaving;
  const activeProfile = preferences.profiles.find((profile) => profile.id === preferences.activeProfileId);
  const runtimeProfileId = useRuntimeStore((state) => state.status?.activeProfileId ?? null);
  const runtimeProfile = preferences.profiles.find((profile) => profile.id === runtimeProfileId);

  useEffect(() => {
    setPreferences(initialPreferences);
    setAppProfileId(initialPreferences.activeProfileId);
  }, [initialPreferences]);

  async function save() {
    if (!canSave) return;
    setIsSaving(true);
    setSaveState("Saving preferences…");
    try {
      const saved = await savePreferences(preferences);
      setSaveState(saved ? "Preferences saved on this PC." : "Preferences could not be saved.");
    } finally {
      setIsSaving(false);
    }
  }

  async function saveProfileUpdate(next: UserPreferences, message: string): Promise<boolean> {
    if (!canSave) return false;
    setIsSaving(true);
    setSaveState("Saving profile…");
    try {
      const saved = await savePreferences(next);
      setSaveState(saved ? message : "Profile could not be saved. Your previous settings are still available.");
      return saved;
    } finally {
      setIsSaving(false);
    }
  }

  async function selectProfile(profile: ControlProfile) {
    if (await saveProfileUpdate(withProfileSettings(preferences, profile), `${profile.name} profile selected.`)) {
      setRenameValue("");
    }
  }

  async function createProfile() {
    try {
      const next = createControlProfile(preferences, newProfileName, crypto.randomUUID());
      const name = newProfileName.trim();
      if (await saveProfileUpdate(next, `${name} profile created.`)) {
      setNewProfileName("");
      setRenameValue("");
      }
    } catch (error) {
      setSaveState(error instanceof Error ? error.message : "Profile could not be created.");
    }
  }

  async function renameProfile() {
    if (!activeProfile || activeProfile.builtIn) return;
    try {
      const name = renameValue.trim();
      if (await saveProfileUpdate(renameControlProfile(preferences, activeProfile.id, name), `Profile renamed to ${name}.`)) setRenameValue("");
    } catch (error) {
      setSaveState(error instanceof Error ? error.message : "Profile could not be renamed.");
    }
  }

  async function deleteProfile() {
    if (!activeProfile || activeProfile.builtIn) return;
    try {
      await saveProfileUpdate(deleteControlProfile(preferences, activeProfile.id), `${activeProfile.name} profile deleted.`);
    } catch (error) {
      setSaveState(error instanceof Error ? error.message : "Profile could not be deleted.");
    }
  }

  async function restoreProfile() {
    if (!activeProfile) return;
    try {
      await saveProfileUpdate(restoreControlProfile(preferences, activeProfile.id), `${activeProfile.name} settings restored.`);
    } catch (error) {
      setSaveState(error instanceof Error ? error.message : "Preset could not be restored.");
    }
  }

  function updateCalibration(edge: "left" | "top" | "right" | "bottom", value: number) {
    const bounds = moveCalibrationEdge(preferences.cursorCalibration, edge, value);
    setPreferences({ ...preferences, cursorCalibration: bounds });
  }

  function resetCalibration() {
    setPreferences({
      ...preferences,
      cursorCalibration: DEFAULT_CURSOR_CALIBRATION,
    });
  }

  async function saveAppRule() {
    try {
      const next = upsertAppProfileRule(preferences, appExecutable, appProfileId);
      if (await saveProfileUpdate(next, `${normalizeExecutableIdentity(appExecutable)} will use the selected profile when its control mode starts.`)) {
        setAppExecutable("");
      }
    } catch (error) {
      setSaveState(error instanceof Error ? error.message : "Application profile could not be saved.");
    }
  }

  async function removeAppRule(executable: string) {
    await saveProfileUpdate(removeAppProfileRule(preferences, executable), `Automatic profile removed for ${executable}.`);
  }

  return (
    <section className="settings-panel">
      <div className="settings-intro"><span className="planned-icon"><Settings2 size={20} aria-hidden="true" /></span><div><h2>Control preferences</h2><p>Preferences are stored locally. System control remains off when AirControl starts.</p></div></div>
      {!runtimeAvailable && <RuntimeUnavailable />}
      <div className="profile-settings">
        <div className="profile-settings-heading"><div><strong>Control profiles</strong><small>Profiles store gesture sensitivity and behavior on this PC. Selecting one never starts system control.</small></div><span>{runtimeProfile ? `${runtimeProfile.name} · active` : activeProfile?.name ?? "No profile"}</span></div>
        <div className="profile-choice-list" role="group" aria-label="Control profile">
          {preferences.profiles.map((profile) => <button type="button" key={profile.id} className={`profile-choice${profile.id === preferences.activeProfileId ? " selected" : ""}`} aria-label={`${profile.name}, ${profile.builtIn ? "built-in preset" : "custom profile"}`} aria-pressed={profile.id === preferences.activeProfileId} disabled={!canSave} onClick={() => void selectProfile(profile)}><strong>{profile.name}</strong><small>{profile.builtIn ? "Built-in preset" : "Your profile"}</small></button>)}
        </div>
        <div className="profile-actions">
          <label className="profile-name-field"><span className="sr-only">New profile name</span><input aria-label="New profile name" maxLength={32} value={newProfileName} disabled={!canSave} placeholder="Name a profile" onChange={(event) => setNewProfileName(event.currentTarget.value)} /></label>
          <button type="button" className="secondary-button" disabled={!canSave || !newProfileName.trim()} onClick={() => void createProfile()}>Save current as profile</button>
          {activeProfile && !activeProfile.builtIn && <>
            <label className="profile-name-field"><span className="sr-only">Rename profile</span><input aria-label="Rename profile" maxLength={32} value={renameValue || activeProfile.name} disabled={!canSave} onChange={(event) => setRenameValue(event.currentTarget.value)} /></label>
            <button type="button" className="secondary-button" disabled={!canSave || !renameValue.trim() || renameValue.trim() === activeProfile.name} onClick={() => void renameProfile()}>Rename</button>
            <button type="button" className="secondary-button danger-button" disabled={!canSave} onClick={() => void deleteProfile()}>Delete</button>
          </>}
          {activeProfile?.builtIn && <button type="button" className="secondary-button" disabled={!canSave} onClick={() => void restoreProfile()}>Restore preset</button>}
        </div>
      </div>
      <div className="profile-settings app-profile-settings">
        <div className="profile-settings-heading"><div><strong>Application profiles</strong><small>Choose an executable-to-profile mapping. Only the executable name is saved; selection occurs when a control mode starts.</small></div></div>
        <label className="app-profile-toggle"><input type="checkbox" checked={preferences.autoProfileEnabled} disabled={!canSave} onChange={(event) => void saveProfileUpdate({ ...preferences, autoProfileEnabled: event.currentTarget.checked }, event.currentTarget.checked ? "Automatic profile selection enabled." : "Automatic profile selection disabled.")} /><span><strong>Automatically select a profile</strong><small>Does not turn on cursor, window, or keyboard control.</small></span></label>
        <div className="app-profile-editor">
          <label className="profile-name-field"><span className="sr-only">Windows executable name</span><input aria-label="Windows executable name" maxLength={260} value={appExecutable} disabled={!canSave} placeholder="Application.exe" onChange={(event) => setAppExecutable(event.currentTarget.value)} /></label>
          <label><span className="sr-only">Profile for application</span><select aria-label="Profile for application" value={appProfileId} disabled={!canSave} onChange={(event) => setAppProfileId(event.currentTarget.value)}>{preferences.profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}</select></label>
          <button type="button" className="secondary-button" disabled={!canSave || !appExecutable.trim() || preferences.appProfileRules.length >= 64 && !preferences.appProfileRules.some((rule) => normalizeExecutableIdentity(rule.executable) === normalizeExecutableIdentity(appExecutable))} onClick={() => void saveAppRule()}>Add or update</button>
        </div>
        {preferences.appProfileRules.length === 0 ? <p className="app-profile-empty">No application mappings yet.</p> : <ul className="app-profile-list">{preferences.appProfileRules.map((rule) => <li key={rule.executable}><code>{rule.executable}</code><span>{preferences.profiles.find((profile) => profile.id === rule.profileId)?.name ?? "Unavailable profile"}</span><select aria-label={`Profile for ${rule.executable}`} value={rule.profileId} disabled={!canSave} onChange={(event) => { try { void saveProfileUpdate(upsertAppProfileRule(preferences, rule.executable, event.currentTarget.value), `${rule.executable} profile updated.`); } catch (error) { setSaveState(error instanceof Error ? error.message : "Application profile could not be saved."); } }}>{preferences.profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}</select><button type="button" className="secondary-button danger-button" disabled={!canSave} aria-label={`Remove ${rule.executable} mapping`} onClick={() => void removeAppRule(rule.executable)}>Remove</button></li>)}</ul>}
      </div>
      <label className="setting-control"><span><strong>Cursor sensitivity</strong><small>Adjust movement gain after calibration.</small></span><span className="range-control"><input aria-label="Cursor sensitivity" type="range" min="0.5" max="2" step="0.1" value={preferences.cursorSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, cursorSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.cursorSensitivity.toFixed(1)}×</output></span></label>
      <label className="setting-control"><span><strong>Cursor smoothing</strong><small>Balance steady motion and responsiveness.</small></span><span className="range-control"><input aria-label="Cursor smoothing" type="range" min="0" max="0.9" step="0.05" value={preferences.cursorSmoothing} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, cursorSmoothing: Number(event.currentTarget.value) })} /><output>{preferences.cursorSmoothing.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Pinch sensitivity</strong><small>Set the normalized pinch threshold.</small></span><span className="range-control"><input aria-label="Pinch sensitivity" type="range" min="0.6" max="1.4" step="0.05" value={preferences.pinchSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, pinchSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.pinchSensitivity.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Window move sensitivity</strong><small>Set the travel distance while moving a window with a pinch.</small></span><span className="range-control"><input aria-label="Window move sensitivity" type="range" min="0.4" max="2" step="0.1" value={preferences.windowSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, windowSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.windowSensitivity.toFixed(1)}×</output></span></label>
      <div className="setting-control calibration-heading"><span><strong>Cursor movement range</strong><small>Map this normalized camera area to the full screen. Keep at least 15% of the frame on each axis.</small></span><button type="button" className="secondary-button" disabled={!canSave} onClick={resetCalibration}>Reset range</button></div>
      <label className="setting-control"><span><strong>Left edge</strong></span><span className="range-control"><input aria-label="Calibration left edge" type="range" min="0" max={preferences.cursorCalibration.right - 0.15} step="0.01" value={preferences.cursorCalibration.left} disabled={!canSave} onChange={(event) => updateCalibration("left", Number(event.currentTarget.value))} /><output>{preferences.cursorCalibration.left.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Top edge</strong></span><span className="range-control"><input aria-label="Calibration top edge" type="range" min="0" max={preferences.cursorCalibration.bottom - 0.15} step="0.01" value={preferences.cursorCalibration.top} disabled={!canSave} onChange={(event) => updateCalibration("top", Number(event.currentTarget.value))} /><output>{preferences.cursorCalibration.top.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Right edge</strong></span><span className="range-control"><input aria-label="Calibration right edge" type="range" min={preferences.cursorCalibration.left + 0.15} max="1" step="0.01" value={preferences.cursorCalibration.right} disabled={!canSave} onChange={(event) => updateCalibration("right", Number(event.currentTarget.value))} /><output>{preferences.cursorCalibration.right.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Bottom edge</strong></span><span className="range-control"><input aria-label="Calibration bottom edge" type="range" min={preferences.cursorCalibration.top + 0.15} max="1" step="0.01" value={preferences.cursorCalibration.bottom} disabled={!canSave} onChange={(event) => updateCalibration("bottom", Number(event.currentTarget.value))} /><output>{preferences.cursorCalibration.bottom.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Scroll sensitivity</strong><small>Adjust scroll distance per hand movement.</small></span><span className="range-control"><input aria-label="Scroll sensitivity" type="range" min="0.4" max="3" step="0.1" value={preferences.scrollSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, scrollSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.scrollSensitivity.toFixed(1)}×</output></span></label>
      <label className="setting-control toggle-setting"><span><strong>Mirror camera coordinates</strong><small>Match cursor direction to a mirrored self-view.</small></span><input aria-label="Mirror camera coordinates" type="checkbox" checked={preferences.cameraMirrored} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, cameraMirrored: event.currentTarget.checked })} /></label>
      <label className="setting-control toggle-setting"><span><strong>Reverse scroll direction</strong><small>Invert vertical hand movement mapping.</small></span><input aria-label="Reverse scroll direction" type="checkbox" checked={preferences.reverseScrollDirection} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, reverseScrollDirection: event.currentTarget.checked })} /></label>
      <label className="setting-control toggle-setting"><span><strong>Enable right click</strong><small>Use a short thumb and middle finger pinch to right-click.</small></span><input aria-label="Enable right click" type="checkbox" checked={preferences.rightClickEnabled} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, rightClickEnabled: event.currentTarget.checked })} /></label>
      <label className="setting-control toggle-setting"><span><strong>Gesture app switching</strong><small>During window control, use a horizontal three-finger swipe to activate the next eligible app window.</small></span><input aria-label="Gesture app switching" type="checkbox" checked={preferences.appSwitchEnabled} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, appSwitchEnabled: event.currentTarget.checked })} /></label>
      <div className="settings-footer"><span role="status">{saveState}</span><button type="button" className="primary-button save-button" disabled={!canSave} onClick={() => void save()}>Save preferences <Check size={14} /></button></div>
    </section>
  );
}

function WindowControlSection({ connection, status }: { connection: "preview" | "connected" | "error"; status: RuntimeSnapshot | null }) {
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [selectedHandle, setSelectedHandle] = useState<string | null>(null);
  const [message, setMessage] = useState("Choose an application window to control.");
  const canUseWindows = connection === "connected";
  const selectedWindow = windows.find((window) => window.handle === selectedHandle);
  const windowModeActive = status?.controlMode === "window";
  const canStartWindowMode = canUseWindows && status?.camera === "running" && status?.handTracking === "detected" && status.controlMode === "disabled" && selectedHandle !== null;

  async function refreshWindows() {
    if (!canUseWindows) return;
    try {
      const result = await invoke<WindowInfo[]>("list_control_windows");
      setWindows(result);
      setSelectedHandle((current) => result.some((window) => window.handle === current) ? current : result[0]?.handle ?? null);
      setMessage(result.length === 0 ? "No eligible application windows were found." : `${result.length} eligible windows found.`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function runWindowAction(command: string, args: Record<string, unknown> = {}) {
    if (!canUseWindows || selectedHandle === null || windowModeActive) return;
    try {
      await invoke(command, { handle: selectedHandle, ...args });
      await refreshWindows();
      setMessage("Window command completed.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function setWindowMoveMode(enabled: boolean) {
    if (!canUseWindows || (enabled && !canStartWindowMode)) return;
    try {
      await invoke("set_window_control", { enabled, handle: enabled ? selectedHandle : null });
      setMessage(enabled ? "Window hand control is on. Pinch with your index finger and move." : "Window hand control is off.");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => { void refreshWindows(); }, [canUseWindows]);

  return (
    <section className="window-control-panel">
      <div className="settings-intro"><span className="planned-icon"><AppWindow size={20} aria-hidden="true" /></span><div><h2>Application windows</h2><p>Only visible, titled, non-tool windows from other processes are listed. Windows may decline activation or movement for protected windows.</p></div></div>
      {!canUseWindows && <RuntimeUnavailable detail="Window enumeration and controls are available in the Windows desktop app." />}
      <div className="window-control-toolbar"><button type="button" className="secondary-button" disabled={!canUseWindows || windowModeActive} onClick={() => void refreshWindows()}><RefreshCw size={14} />Refresh windows</button><button type="button" className={windowModeActive ? "secondary-button" : "primary-button save-button"} disabled={windowModeActive ? !canUseWindows : !canStartWindowMode} onClick={() => void setWindowMoveMode(!windowModeActive)}>{windowModeActive ? "Stop hand move" : "Start hand move"}</button><span role="status">{message}</span></div>
      {windowModeActive && <p className="window-mode-status" role="status">Hand control is active for <strong>{selectedWindow?.title ?? "the selected window"}</strong>. Pinch with one hand to move; spread or bring two tracked hands together to resize. If either hand is lost, resizing stops immediately.</p>}
      {windows.length === 0 ? <div className="window-empty-state"><AppWindow size={22} aria-hidden="true" /><strong>No eligible windows</strong><span>Open a regular desktop app, then refresh the list.</span></div> : <div className="window-control-layout">
        <div className="window-list" role="listbox" aria-label="Available application windows">
          {windows.map((window) => <button type="button" role="option" aria-selected={selectedHandle === window.handle} className={`window-list-item ${selectedHandle === window.handle ? "selected" : ""}`} key={window.handle} disabled={windowModeActive} onClick={() => setSelectedHandle(window.handle)}><strong>{window.title}</strong><small>{window.rect.width} × {window.rect.height} · PID {window.processId}</small></button>)}
        </div>
        {selectedWindow && <div className="window-actions">
          <div><span className="eyebrow">SELECTED WINDOW</span><h3>{selectedWindow.title}</h3><p>{selectedWindow.rect.x}, {selectedWindow.rect.y} · {selectedWindow.rect.width} × {selectedWindow.rect.height}</p></div>
          <div className="window-action-row"><button type="button" className="secondary-button" disabled={windowModeActive} onClick={() => void runWindowAction("focus_control_window")}>Activate</button><button type="button" className="secondary-button" disabled={windowModeActive} onClick={() => void runWindowAction("minimize_control_window")}>Minimize</button><button type="button" className="secondary-button" disabled={windowModeActive} onClick={() => void runWindowAction("maximize_control_window")}>Maximize</button></div>
          <div className="window-action-row"><button type="button" className="secondary-button" aria-label="Move window left" disabled={windowModeActive} onClick={() => void runWindowAction("move_control_window", { delta: [-40, 0] })}>← 40 px</button><button type="button" className="secondary-button" aria-label="Move window up" disabled={windowModeActive} onClick={() => void runWindowAction("move_control_window", { delta: [0, -40] })}>↑ 40 px</button><button type="button" className="secondary-button" aria-label="Move window down" disabled={windowModeActive} onClick={() => void runWindowAction("move_control_window", { delta: [0, 40] })}>↓ 40 px</button><button type="button" className="secondary-button" aria-label="Move window right" disabled={windowModeActive} onClick={() => void runWindowAction("move_control_window", { delta: [40, 0] })}>40 px →</button></div>
          <div className="window-action-row"><button type="button" className="secondary-button" disabled={windowModeActive} onClick={() => void runWindowAction("resize_control_window", { scale: 0.9 })}>Shrink 10%</button><button type="button" className="secondary-button" disabled={windowModeActive} onClick={() => void runWindowAction("resize_control_window", { scale: 1.1 })}>Enlarge 10%</button></div>
        </div>}
      </div>}
    </section>
  );
}

function AirKeyboardSurface() {
  const [feedback, dispatchFeedback] = useReducer(reduceKeyboardFeedback, INITIAL_KEYBOARD_FEEDBACK);
  const { legend, shiftArmed } = feedback;
  const [error, setError] = useState<string | null>(null);
  const [tracking, setTracking] = useState<RuntimeSnapshot["handTracking"]>("waiting");
  const [modeActive, setModeActive] = useState(false);

  useEffect(() => {
    document.body.classList.add("keyboard-surface");
    document.documentElement.classList.add("keyboard-surface");
    return () => {
      document.body.classList.remove("keyboard-surface");
      document.documentElement.classList.remove("keyboard-surface");
    };
  }, []);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let stopRuntime: UnlistenFn | undefined;
    let stopTracking: UnlistenFn | undefined;
    const syncStatus = (snapshot: RuntimeSnapshot) => {
      setTracking(snapshot.handTracking);
      setModeActive(snapshot.controlMode === "airKeyboard");
      if (snapshot.controlMode !== "airKeyboard") {
        dispatchFeedback({ type: "modeStopped" });
      }
    };
    void Promise.all([
      listen<RuntimeSnapshot>("runtime-status", (event) => syncStatus(event.payload)),
      listen<{ state: RuntimeSnapshot["handTracking"] }>("hand-tracking-status", (event) => {
        setTracking(event.payload.state);
        dispatchFeedback({ type: "trackingChanged", tracking: event.payload.state });
      }),
    ]).then(([runtimeListener, trackingListener]) => {
      if (disposed) { runtimeListener(); trackingListener(); }
      else {
        stopRuntime = runtimeListener;
        stopTracking = trackingListener;
        void invoke<RuntimeSnapshot>("get_runtime_status").then((snapshot) => {
          if (!disposed) syncStatus(snapshot);
        }).catch(() => undefined);
      }
    });
    return () => { disposed = true; stopRuntime?.(); stopTracking?.(); };
  }, []);

  async function press(key: KeyboardKey) {
    if (!isTauri()) return;
    try {
      await invoke("press_air_keyboard_key", { key: commandKey(key) });
      dispatchFeedback({ type: "keyAccepted", keyId: key.id });
      if (key.id !== "shift") setError(null);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : "Key input was rejected.");
    }
  }

  async function stopKeyboard() {
    if (isTauri()) await invoke("set_air_keyboard_control", { enabled: false });
  }

  return (
    <main className="air-keyboard-window" aria-label="Air Keyboard">
      <header className="air-keyboard-header" data-tauri-drag-region><div data-tauri-drag-region><span className="keyboard-live-dot" /><strong>Air Keyboard</strong><span className="keyboard-layout-label">Key labels · {legend === "korean" ? "Korean Dubeolsik" : "English QWERTY"}</span></div><button type="button" className="keyboard-close" aria-label="Stop Air Keyboard" onClick={() => void stopKeyboard()}>×</button></header>
      <div className="air-keyboard-rows" role="group" aria-label={`${legend} keyboard key labels`}>
        {KEYBOARD_ROWS.map((row, rowIndex) => <div className="air-keyboard-row" key={rowIndex}>
          {row.map((key) => <button type="button" key={key.id} data-key={key.id} className={`air-key ${key.kind ?? "letter"} ${key.id === "shift" && shiftArmed ? "armed" : ""}`} style={{ flexGrow: key.width ?? 1 }} aria-label={key.id === "layout" ? "Toggle English or Korean key labels" : key.label} aria-pressed={key.id === "shift" ? shiftArmed : undefined} disabled={!modeActive} onClick={() => void press(key)}>
            <span>{legend === "korean" && key.korean ? key.korean : key.label}</span>
            {key.korean && legend === "english" && <small>{key.korean}</small>}
            {key.shifted && <small className="shifted-legend">{key.shifted}</small>}
          </button>)}
        </div>)}
      </div>
      <footer className="air-keyboard-footer"><span>{tracking === "detected" ? "Point with your index finger · pinch briefly to type" : "Hand tracking paused · waiting for a confident hand pose"}</span><span>Text is sent directly to the active app and is never stored. The key-label choice does not report the target app’s IME state.</span></footer>
      {error && <p className="air-keyboard-error" role="alert">{error}</p>}
    </main>
  );
}

function AirKeyboardSection({ connection, status }: { connection: "preview" | "connected" | "error"; status: RuntimeSnapshot | null }) {
  const [message, setMessage] = useState("");
  const enabled = status?.controlMode === "airKeyboard";
  const canEnable = connection === "connected" && status?.camera === "running" && status?.handTracking === "detected" && status.controlMode === "disabled";
  async function toggle() {
    if (!isTauri() || (!enabled && !canEnable)) return;
    try {
      await invoke("set_air_keyboard_control", { enabled: !enabled });
      setMessage(enabled ? "Air Keyboard stopped and temporary modifier state was cleared." : "The panel is configured not to take keyboard focus. The target app’s IME state is not inspected.");
    } catch (error) { setMessage(error instanceof Error ? error.message : String(error)); }
  }
  return <section className="keyboard-control-panel" aria-label="Air Keyboard controls">
    <div className="settings-intro"><span className="planned-icon"><Keyboard size={20} aria-hidden="true" /></span><div><h2>Type with hand gestures</h2><p>Enable the camera and wait for hand tracking. Point at a key, then make a brief thumb and index pinch.</p></div></div>
    <div className="keyboard-mode-row"><span className={`keyboard-mode-pill ${enabled ? "active" : ""}`}><span />{enabled ? "Keyboard control active" : "Keyboard control off"}</span><button type="button" className={enabled ? "secondary-button" : "primary-button save-button"} disabled={!enabled && !canEnable} onClick={() => void toggle()}>{enabled ? "Stop Air Keyboard" : "Start Air Keyboard"}</button></div>
    {connection !== "connected" && <RuntimeUnavailable detail="Air Keyboard input is available in the Windows desktop app." />}
    {!enabled && connection === "connected" && status?.camera !== "running" && <p className="keyboard-guidance">Start the camera from Camera &amp; tracking first. Hand control remains off until you start it here.</p>}
    {enabled && <p className="window-mode-status">Aim the hand cursor at the floating keyboard. Short pinches click once; a long pinch is ignored. Stop the camera or this mode to end input.</p>}
    <div className="keyboard-preview" aria-label="Keyboard preview"><span className="eyebrow">LAYOUT PREVIEW</span>{KEYBOARD_ROWS.slice(0, 3).map((row, index) => <div className="keyboard-preview-row" key={index}>{row.map((key) => <span key={key.id}>{key.label}</span>)}</div>)}</div>
    <p className="keyboard-privacy-note">No typed characters are logged or saved. English/Korean selects the key legends and sends the Windows Hangul key. AirControl does not read the target app’s current IME state; use that app’s input indicator to confirm it.</p>
    <p className="keyboard-status" role="status">{message}</p>
  </section>;
}

function App() {
  if (new URLSearchParams(window.location.search).get("surface") === "keyboard") return <AirKeyboardSurface />;
  return <DashboardApp />;
}

function DashboardApp() {
  const [activeSection, setActiveSection] = useState("Overview");
  const { status, preferences, connection, error, refresh, savePreferences, listCameras, startCamera, stopCamera, setCursorControl, setRuntimeStatus, setCameraState, setHandTrackingState } = useRuntimeStore();

  useEffect(() => { void refresh(); }, [refresh]);
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let unlisten: UnlistenFn | undefined;
    void listen<RuntimeSnapshot>("runtime-status", (event) => setRuntimeStatus(event.payload)).then((stopListening) => {
      if (disposed) stopListening();
      else unlisten = stopListening;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [setRuntimeStatus]);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup"><div className="brand-mark"><MousePointer2 size={19} strokeWidth={2.4} /></div><div><strong>aircontrol</strong><small>WINDOWS PREVIEW</small></div></div>
        <div className="workspace-switch"><span className="workspace-avatar">A</span><span><strong>Personal workspace</strong><small>Local profile</small></span><ChevronRight size={14} className="workspace-chevron" /></div>
        {(["Workspace", "Manage"] as const).map((group) => (
          <nav aria-label={group} className="nav-group" key={group}>
            <span className="nav-label">{group}</span>
            {navigation.filter((item) => item.group === group).map(({ label, icon: Icon }) => (
              <button key={label} type="button" className={`nav-item ${activeSection === label ? "active" : ""}`} onClick={() => setActiveSection(label)} aria-current={activeSection === label ? "page" : undefined}>
                <Icon size={17} strokeWidth={1.8} aria-hidden="true" /><span>{label}</span>{label === "Overview" && <span className="nav-indicator" />}
              </button>
            ))}
          </nav>
        ))}
        <div className="sidebar-spacer" />
        <div className="privacy-card"><div><ShieldCheck size={16} /><strong>Private by design</strong></div><p>Camera frames stay on this PC. AirControl sends no video to a server.</p></div>
        <div className="profile-row"><span className="profile-avatar">D</span><span><strong>dltldn1234</strong><small>Personal edition</small></span><button type="button" aria-label="Profile options" className="profile-menu">•••</button></div>
      </aside>

      <main className="main-area">
        <header className="topbar"><div className="breadcrumbs"><span>AirControl</span><ChevronRight size={14} /><strong>{activeSection}</strong></div><div className="topbar-actions"><span className={`runtime-chip ${connection}`}><span />{connection === "connected" ? "Runtime connected" : "Preview mode"}</span><button className="icon-button" type="button" aria-label="Keyboard shortcuts"><Command size={16} /></button><span className="topbar-avatar">D</span></div></header>
        <div className="content-scroll">
          <div className="page-title-row"><div><div className="eyebrow page-eyebrow">{activeSection === "Overview" ? "YOUR WORKSPACE" : "AIRCONTROL"}</div><h1>{activeSection}</h1><p>{sectionDescriptions[activeSection]}</p></div><span className="phase-badge">{activeSection === "Overview" ? "LOCAL FIRST" : "IN DEVELOPMENT"}</span></div>
          {activeSection === "Overview" ? <Overview status={status} connection={connection} error={error} setCursorControl={setCursorControl} /> : activeSection === "Settings" ? <SettingsSection connection={connection} initialPreferences={preferences} savePreferences={savePreferences} /> : activeSection === "Camera & tracking" ? <CameraSection connection={connection} cameraState={status?.camera ?? "unavailable"} handTrackingState={status?.handTracking ?? "waiting"} error={error} listCameras={listCameras} startCamera={startCamera} stopCamera={stopCamera} setCameraState={setCameraState} setHandTrackingState={setHandTrackingState} /> : activeSection === "Window control" ? <WindowControlSection connection={connection} status={status} /> : activeSection === "Air Keyboard" ? <AirKeyboardSection connection={connection} status={status} /> : activeSection === "Command palette" ? <CommandPaletteSection connection={connection} status={status} /> : <PlannedSection title={activeSection} />}
          <footer className="page-footer"><span>AirControl for Windows <span className="footer-divider">·</span> Foundation build</span><span>Local processing <span className="footer-divider">·</span> No cloud connection</span></footer>
        </div>
      </main>
    </div>
  );
}

export default App;
