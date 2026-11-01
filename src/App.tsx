import {
  Activity,
  AppWindow,
  ArrowUpRight,
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
  WandSparkles,
} from "lucide-react";
import { useEffect, useState } from "react";
import { useRuntimeStore } from "./state/runtimeStore";
import type { RuntimeSnapshot } from "./domain/runtimeStatus";
import type { UserPreferences } from "./domain/userPreferences";
import { cameraLabel, gestureLabel, trackingLabel } from "./domain/runtimeStatus";

const navigation = [
  { label: "Overview", icon: Activity, group: "Workspace" },
  { label: "Camera & tracking", icon: Camera, group: "Workspace" },
  { label: "Gestures", icon: Hand, group: "Workspace" },
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

function RuntimeUnavailable() {
  return (
    <div className="runtime-notice" role="status">
      <span className="notice-icon"><CircleHelp size={17} aria-hidden="true" /></span>
      <div>
        <strong>Desktop runtime is not connected</strong>
        <p>Live camera and system status will appear here when AirControl runs as a Tauri desktop app.</p>
      </div>
    </div>
  );
}

function Overview({ status }: { status: RuntimeSnapshot | null }) {
  const camera = status?.camera ?? "unavailable";
  const tracking = status?.handTracking ?? "waiting";
  const mode = status?.controlMode ?? "disabled";
  const gesture = status?.activeGesture ?? "none";
  const fps = status?.framesPerSecond;

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
        <StatusCard icon={Hand} label="HAND TRACKING" value={trackingLabel(tracking)} detail={tracking === "detected" ? "Landmarks detected" : "No hand data received"} />
        <StatusCard icon={MousePointer2} label="CONTROL MODE" value={mode === "disabled" ? "Off" : mode} detail={mode === "disabled" ? "No system input is being sent" : "Explicitly enabled"} />
        <StatusCard icon={WandSparkles} label="GESTURE" value={gestureLabel(gesture)} detail={fps == null ? "FPS unavailable" : `${fps.toFixed(0)} frames per second`} />
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
  const canSave = connection === "connected";

  useEffect(() => setPreferences(initialPreferences), [initialPreferences]);

  async function save() {
    setSaveState("Saving preferences…");
    const saved = await savePreferences(preferences);
    setSaveState(saved ? "Preferences saved on this PC." : "Preferences could not be saved in preview mode.");
  }

  return (
    <section className="settings-panel">
      <div className="settings-intro"><span className="planned-icon"><Settings2 size={20} aria-hidden="true" /></span><div><h2>Control preferences</h2><p>Preferences are stored locally. System control remains off when AirControl starts.</p></div></div>
      {!canSave && <RuntimeUnavailable />}
      <label className="setting-control"><span><strong>Cursor sensitivity</strong><small>Adjust movement gain after calibration.</small></span><span className="range-control"><input aria-label="Cursor sensitivity" type="range" min="0.5" max="2" step="0.1" value={preferences.cursorSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, cursorSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.cursorSensitivity.toFixed(1)}×</output></span></label>
      <label className="setting-control"><span><strong>Cursor smoothing</strong><small>Balance steady motion and responsiveness.</small></span><span className="range-control"><input aria-label="Cursor smoothing" type="range" min="0" max="0.9" step="0.05" value={preferences.cursorSmoothing} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, cursorSmoothing: Number(event.currentTarget.value) })} /><output>{preferences.cursorSmoothing.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Pinch sensitivity</strong><small>Set the normalized pinch threshold.</small></span><span className="range-control"><input aria-label="Pinch sensitivity" type="range" min="0.6" max="1.4" step="0.05" value={preferences.pinchSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, pinchSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.pinchSensitivity.toFixed(2)}</output></span></label>
      <label className="setting-control"><span><strong>Scroll sensitivity</strong><small>Adjust scroll distance per hand movement.</small></span><span className="range-control"><input aria-label="Scroll sensitivity" type="range" min="0.4" max="3" step="0.1" value={preferences.scrollSensitivity} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, scrollSensitivity: Number(event.currentTarget.value) })} /><output>{preferences.scrollSensitivity.toFixed(1)}×</output></span></label>
      <label className="setting-control toggle-setting"><span><strong>Mirror camera coordinates</strong><small>Match cursor direction to a mirrored self-view.</small></span><input aria-label="Mirror camera coordinates" type="checkbox" checked={preferences.cameraMirrored} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, cameraMirrored: event.currentTarget.checked })} /></label>
      <label className="setting-control toggle-setting"><span><strong>Reverse scroll direction</strong><small>Invert vertical hand movement mapping.</small></span><input aria-label="Reverse scroll direction" type="checkbox" checked={preferences.reverseScrollDirection} disabled={!canSave} onChange={(event) => setPreferences({ ...preferences, reverseScrollDirection: event.currentTarget.checked })} /></label>
      <div className="settings-footer"><span role="status">{saveState}</span><button type="button" className="primary-button save-button" disabled={!canSave} onClick={() => void save()}>Save preferences <Check size={14} /></button></div>
    </section>
  );
}

export default function App() {
  const [activeSection, setActiveSection] = useState("Overview");
  const { status, preferences, connection, refresh, savePreferences } = useRuntimeStore();

  useEffect(() => { void refresh(); }, [refresh]);

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
          <div className="page-title-row"><div><div className="eyebrow page-eyebrow">{activeSection === "Overview" ? "YOUR WORKSPACE" : "AIRCONTROL"}</div><h1>{activeSection}</h1><p>{sectionDescriptions[activeSection]}</p></div>{activeSection === "Overview" ? <button className="primary-button" type="button" disabled title="Camera and control services are not available yet"><span className="button-status" /> Start control <ArrowUpRight size={15} /></button> : <span className="phase-badge">FOUNDATION PHASE</span>}</div>
          {activeSection === "Overview" ? <Overview status={status} /> : activeSection === "Settings" ? <SettingsSection connection={connection} initialPreferences={preferences} savePreferences={savePreferences} /> : <PlannedSection title={activeSection} />}
          <footer className="page-footer"><span>AirControl for Windows <span className="footer-divider">·</span> Foundation build</span><span>Local processing <span className="footer-divider">·</span> No cloud connection</span></footer>
        </div>
      </main>
    </div>
  );
}
