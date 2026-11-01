use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const PALETTE_COMMANDS: [PaletteCommand; 6] = [
    PaletteCommand::NextApplication,
    PaletteCommand::SnapWindowLeft,
    PaletteCommand::SnapWindowRight,
    PaletteCommand::PlayPause,
    PaletteCommand::VolumeDown,
    PaletteCommand::VolumeUp,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PaletteCommand {
    NextApplication,
    SnapWindowLeft,
    SnapWindowRight,
    PlayPause,
    VolumeDown,
    VolumeUp,
}

impl PaletteCommand {
    pub const fn id(self) -> &'static str {
        match self {
            Self::NextApplication => "nextApplication",
            Self::SnapWindowLeft => "snapWindowLeft",
            Self::SnapWindowRight => "snapWindowRight",
            Self::PlayPause => "playPause",
            Self::VolumeDown => "volumeDown",
            Self::VolumeUp => "volumeUp",
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        PALETTE_COMMANDS.get(index).copied()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        PALETTE_COMMANDS
            .iter()
            .copied()
            .find(|command| command.id() == id)
    }
}

pub fn command_for_active_palette(
    mode: crate::ControlMode,
    id: &str,
) -> Result<PaletteCommand, String> {
    if mode != crate::ControlMode::CommandPalette {
        return Err("Open the command palette before running a command.".to_owned());
    }
    PaletteCommand::from_id(id)
        .ok_or_else(|| "Choose a command from the AirControl allowlist.".to_owned())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaletteUpdate {
    pub active: bool,
    pub hovered_index: Option<usize>,
    pub command: Option<PaletteCommand>,
    pub cancelled: bool,
}

#[derive(Clone, Copy, Debug)]
enum PinchState {
    Open,
    Closed { started_at: Instant },
}

/// Explicitly opened, exclusive hand-selection mode. The command grid is
/// divided into three columns and two rows in normalized camera coordinates.
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteStateMachine {
    active: bool,
    hovered_index: Option<usize>,
    pinch_state: PinchState,
    last_confirmed_at: Option<Instant>,
    fist_started_at: Option<Instant>,
}

impl Default for CommandPaletteStateMachine {
    fn default() -> Self {
        Self {
            active: false,
            hovered_index: None,
            pinch_state: PinchState::Open,
            last_confirmed_at: None,
            fist_started_at: None,
        }
    }
}

impl CommandPaletteStateMachine {
    pub fn is_active(self) -> bool {
        self.active
    }

    pub fn open(&mut self) -> PaletteUpdate {
        self.reset();
        self.active = true;
        self.snapshot(None, false)
    }

    pub fn update(
        &mut self,
        position: Option<[f32; 2]>,
        pinch_distance: Option<f32>,
        cancel_pose: bool,
        now: Instant,
    ) -> PaletteUpdate {
        if !self.active {
            return self.snapshot(None, false);
        }
        let Some(position) = position.filter(|point| {
            point
                .iter()
                .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        }) else {
            return self.cancel();
        };

        if cancel_pose {
            let started = *self.fist_started_at.get_or_insert(now);
            if now.saturating_duration_since(started) >= Duration::from_millis(650) {
                return self.cancel();
            }
            self.hovered_index = None;
            self.pinch_state = PinchState::Open;
            return self.snapshot(None, false);
        }
        self.fist_started_at = None;

        let column = ((position[0] * 3.0).floor() as usize).min(2);
        let row = ((position[1] * 2.0).floor() as usize).min(1);
        self.hovered_index = Some(row * 3 + column);

        let Some(distance) = pinch_distance.filter(|value| value.is_finite() && *value >= 0.0)
        else {
            return self.cancel();
        };
        let command = self.update_pinch(distance, now);
        self.snapshot(command, false)
    }

    pub fn tracking_lost(&mut self) -> PaletteUpdate {
        self.cancel()
    }

    pub fn mode_changed(&mut self) -> PaletteUpdate {
        self.cancel()
    }

    fn update_pinch(&mut self, distance: f32, now: Instant) -> Option<PaletteCommand> {
        let start_threshold = 0.28;
        let release_threshold = 0.42;
        let cooldown_ready = self
            .last_confirmed_at
            .is_none_or(|last| now.saturating_duration_since(last) >= Duration::from_millis(450));
        match self.pinch_state {
            PinchState::Open if distance <= start_threshold && cooldown_ready => {
                self.pinch_state = PinchState::Closed { started_at: now };
                None
            }
            PinchState::Open => None,
            PinchState::Closed { started_at } if distance >= release_threshold => {
                self.pinch_state = PinchState::Open;
                let elapsed = now.saturating_duration_since(started_at);
                if elapsed <= Duration::from_millis(500) {
                    let command = self.hovered_index.and_then(PaletteCommand::from_index);
                    if command.is_some() {
                        self.last_confirmed_at = Some(now);
                    }
                    command
                } else {
                    None
                }
            }
            PinchState::Closed { .. } => None,
        }
    }

    fn cancel(&mut self) -> PaletteUpdate {
        let was_active = self.active;
        self.reset();
        self.snapshot(None, was_active)
    }

    fn reset(&mut self) {
        self.active = false;
        self.hovered_index = None;
        self.pinch_state = PinchState::Open;
        self.fist_started_at = None;
    }

    fn snapshot(&self, command: Option<PaletteCommand>, cancelled: bool) -> PaletteUpdate {
        PaletteUpdate {
            active: self.active,
            hovered_index: self.hovered_index,
            command,
            cancelled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommandPaletteStateMachine, PALETTE_COMMANDS, PaletteCommand, command_for_active_palette,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn palette_requires_explicit_open_and_exposes_a_fixed_allowlist() {
        let mut palette = CommandPaletteStateMachine::default();
        assert!(!palette.is_active());
        assert_eq!(PALETTE_COMMANDS.len(), 6);
        assert_eq!(PaletteCommand::from_index(6), None);
        assert!(palette.open().active);
    }

    #[test]
    fn hover_maps_normalized_positions_to_the_six_command_slots() {
        let mut palette = CommandPaletteStateMachine::default();
        palette.open();
        let start = Instant::now();
        for (position, expected) in [
            ([0.1, 0.1], 0),
            ([0.5, 0.1], 1),
            ([0.9, 0.1], 2),
            ([0.1, 0.9], 3),
            ([0.5, 0.9], 4),
            ([0.9, 0.9], 5),
        ] {
            assert_eq!(
                palette
                    .update(Some(position), Some(0.8), false, start)
                    .hovered_index,
                Some(expected)
            );
        }
    }

    #[test]
    fn short_pinch_confirms_hovered_command_once_after_release() {
        let start = Instant::now();
        let mut palette = CommandPaletteStateMachine::default();
        palette.open();
        palette.update(Some([0.1, 0.1]), Some(0.8), false, start);
        palette.update(Some([0.1, 0.1]), Some(0.2), false, start);
        let confirm = palette.update(
            Some([0.1, 0.1]),
            Some(0.5),
            false,
            start + Duration::from_millis(120),
        );

        assert_eq!(confirm.command, Some(PaletteCommand::NextApplication));
        assert_eq!(
            palette
                .update(
                    Some([0.1, 0.1]),
                    Some(0.8),
                    false,
                    start + Duration::from_millis(130)
                )
                .command,
            None
        );
    }

    #[test]
    fn tracking_loss_and_mode_changes_cancel_and_clear_selection() {
        let mut palette = CommandPaletteStateMachine::default();
        palette.open();
        palette.update(Some([0.5, 0.1]), Some(0.8), false, Instant::now());

        let cancelled = palette.tracking_lost();

        assert!(!cancelled.active);
        assert!(cancelled.cancelled);
        assert_eq!(cancelled.hovered_index, None);

        palette.open();
        palette.update(Some([0.5, 0.1]), Some(0.8), false, Instant::now());
        let mode_changed = palette.mode_changed();
        assert!(!mode_changed.active);
        assert!(mode_changed.cancelled);
        assert_eq!(mode_changed.hovered_index, None);
    }

    #[test]
    fn held_fist_cancels_without_dispatching_a_command() {
        let start = Instant::now();
        let mut palette = CommandPaletteStateMachine::default();
        palette.open();
        palette.update(Some([0.1, 0.1]), Some(0.2), false, start);
        palette.update(
            Some([0.1, 0.1]),
            Some(0.2),
            true,
            start + Duration::from_millis(10),
        );

        let cancelled = palette.update(
            Some([0.1, 0.1]),
            Some(0.2),
            true,
            start + Duration::from_millis(700),
        );

        assert!(!cancelled.active);
        assert!(cancelled.cancelled);
        assert_eq!(cancelled.command, None);
    }

    #[test]
    fn invalid_hand_samples_cancel_the_palette() {
        let mut palette = CommandPaletteStateMachine::default();
        palette.open();

        assert!(
            palette
                .update(Some([f32::NAN, 0.2]), Some(0.1), false, Instant::now())
                .cancelled
        );
        assert!(!palette.is_active());
    }

    #[test]
    fn command_dispatch_requires_active_palette_and_a_known_command() {
        assert_eq!(
            command_for_active_palette(crate::ControlMode::CommandPalette, "playPause"),
            Ok(PaletteCommand::PlayPause)
        );
        assert!(command_for_active_palette(crate::ControlMode::Cursor, "playPause").is_err());
        assert!(
            command_for_active_palette(crate::ControlMode::CommandPalette, "runShell").is_err()
        );
    }
}
