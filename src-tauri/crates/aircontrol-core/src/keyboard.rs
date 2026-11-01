use std::time::{Duration, Instant};

pub const VK_HANGUL: u16 = 0x15;
pub const VK_BACK: u16 = 0x08;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_SPACE: u16 = 0x20;
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CAPITAL: u16 = 0x14;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyboardBounds {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

impl KeyboardBounds {
    pub const fn is_usable(self) -> bool {
        self.width > 0 && self.height > 0
    }

    pub fn contains(self, point: [i32; 2]) -> bool {
        if !self.is_usable() {
            return false;
        }
        let x = i64::from(point[0]) - i64::from(self.left);
        let y = i64::from(point[1]) - i64::from(self.top);
        x >= 0 && y >= 0 && x < i64::from(self.width) && y < i64::from(self.height)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum KeyboardLayout {
    #[default]
    English,
    Korean,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyStroke {
    pub virtual_key: u16,
    pub shift: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct KeyboardSession {
    shift_armed: bool,
    layout: KeyboardLayout,
}

impl KeyboardSession {
    pub fn layout(self) -> KeyboardLayout {
        self.layout
    }

    pub fn shift_armed(self) -> bool {
        self.shift_armed
    }

    pub fn reset(&mut self) {
        self.shift_armed = false;
        self.layout = KeyboardLayout::English;
    }

    pub fn reset_modifiers(&mut self) {
        self.shift_armed = false;
    }

    pub fn press(&mut self, key: &str) -> Result<Option<KeyStroke>, String> {
        match key {
            "shift" => {
                self.shift_armed = !self.shift_armed;
                Ok(None)
            }
            "caps" => Ok(Some(KeyStroke {
                virtual_key: VK_CAPITAL,
                shift: false,
            })),
            "layout" => {
                self.layout = match self.layout {
                    KeyboardLayout::English => KeyboardLayout::Korean,
                    KeyboardLayout::Korean => KeyboardLayout::English,
                };
                Ok(Some(KeyStroke {
                    virtual_key: VK_HANGUL,
                    shift: false,
                }))
            }
            "space" => self.character(VK_SPACE, false),
            "enter" => self.character(VK_RETURN, false),
            "backspace" => self.character(VK_BACK, false),
            _ => {
                let (virtual_key, needs_shift, _is_letter) = map_character(key)
                    .ok_or_else(|| "This key is not part of the Air Keyboard layout.".to_owned())?;
                let shift = self.shift_armed || needs_shift;
                self.shift_armed = false;
                Ok(Some(KeyStroke { virtual_key, shift }))
            }
        }
    }

    fn character(&mut self, virtual_key: u16, shift: bool) -> Result<Option<KeyStroke>, String> {
        let use_shift = shift || self.shift_armed;
        self.shift_armed = false;
        Ok(Some(KeyStroke {
            virtual_key,
            shift: use_shift,
        }))
    }
}

fn map_character(key: &str) -> Option<(u16, bool, bool)> {
    if key.len() == 1 {
        let character = key.as_bytes()[0];
        return match character {
            b'a'..=b'z' => Some((u16::from(character.to_ascii_uppercase()), false, true)),
            b'A'..=b'Z' => Some((u16::from(character), false, true)),
            b'0'..=b'9' => Some((u16::from(character), false, false)),
            b' ' => Some((VK_SPACE, false, false)),
            b'-' => Some((0xBD, false, false)),
            b'_' => Some((0xBD, true, false)),
            b'=' => Some((0xBB, false, false)),
            b'+' => Some((0xBB, true, false)),
            b'[' => Some((0xDB, false, false)),
            b'{' => Some((0xDB, true, false)),
            b']' => Some((0xDD, false, false)),
            b'}' => Some((0xDD, true, false)),
            b';' => Some((0xBA, false, false)),
            b':' => Some((0xBA, true, false)),
            b'\'' => Some((0xDE, false, false)),
            b'"' => Some((0xDE, true, false)),
            b',' => Some((0xBC, false, false)),
            b'<' => Some((0xBC, true, false)),
            b'.' => Some((0xBE, false, false)),
            b'>' => Some((0xBE, true, false)),
            b'/' => Some((0xBF, false, false)),
            b'?' => Some((0xBF, true, false)),
            b'`' => Some((0xC0, false, false)),
            b'~' => Some((0xC0, true, false)),
            _ => None,
        };
    }
    None
}

#[derive(Clone, Copy, Debug)]
enum TapState {
    Idle,
    Pinching { started_at: Instant },
    BlockedUntilRelease,
}

#[derive(Clone, Copy, Debug)]
pub struct KeyboardTapRecognizer {
    state: TapState,
    last_tap_at: Option<Instant>,
    cooldown: Duration,
    maximum_tap: Duration,
}

impl Default for KeyboardTapRecognizer {
    fn default() -> Self {
        Self {
            state: TapState::Idle,
            last_tap_at: None,
            cooldown: Duration::from_millis(350),
            maximum_tap: Duration::from_millis(700),
        }
    }
}

impl KeyboardTapRecognizer {
    pub fn update(&mut self, distance: Option<f32>, now: Instant) -> bool {
        let Some(distance) = distance.filter(|value| value.is_finite()) else {
            self.reset();
            return false;
        };
        match self.state {
            TapState::Idle => {
                let ready = self
                    .last_tap_at
                    .is_none_or(|last| now.saturating_duration_since(last) >= self.cooldown);
                if ready && distance <= 0.28 {
                    self.state = TapState::Pinching { started_at: now };
                }
                false
            }
            TapState::Pinching { started_at } => {
                if distance >= 0.42 {
                    self.state = TapState::Idle;
                    self.last_tap_at = Some(now);
                    now.saturating_duration_since(started_at) <= self.maximum_tap
                } else if now.saturating_duration_since(started_at) > self.maximum_tap {
                    self.state = TapState::BlockedUntilRelease;
                    false
                } else {
                    false
                }
            }
            TapState::BlockedUntilRelease => {
                if distance >= 0.42 {
                    self.state = TapState::Idle;
                    self.last_tap_at = Some(now);
                }
                false
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = TapState::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qwerty_keys_map_to_windows_virtual_keys() {
        let mut keyboard = KeyboardSession::default();
        assert_eq!(
            keyboard.press("a").unwrap(),
            Some(KeyStroke {
                virtual_key: b'A' as u16,
                shift: false
            })
        );
        assert_eq!(
            keyboard.press("1").unwrap(),
            Some(KeyStroke {
                virtual_key: b'1' as u16,
                shift: false
            })
        );
        assert_eq!(
            keyboard.press("?").unwrap(),
            Some(KeyStroke {
                virtual_key: 0xBF,
                shift: true
            })
        );
        assert!(keyboard.press("not-a-key").is_err());
    }

    #[test]
    fn shift_is_one_shot_and_caps_lock_only_sends_a_windows_toggle_key() {
        let mut keyboard = KeyboardSession::default();
        keyboard.press("shift").unwrap();
        assert!(keyboard.press("a").unwrap().unwrap().shift);
        assert!(!keyboard.shift_armed());
        assert_eq!(
            keyboard.press("caps").unwrap(),
            Some(KeyStroke {
                virtual_key: VK_CAPITAL,
                shift: false
            })
        );
        assert!(!keyboard.press("a").unwrap().unwrap().shift);
        keyboard.press("shift").unwrap();
        assert!(keyboard.press("a").unwrap().unwrap().shift);
    }

    #[test]
    fn tracking_loss_clears_one_shot_shift_without_resetting_the_layout() {
        let mut keyboard = KeyboardSession::default();
        keyboard.press("layout").unwrap();
        keyboard.press("shift").unwrap();

        keyboard.reset_modifiers();

        assert!(!keyboard.shift_armed());
        assert_eq!(keyboard.layout(), KeyboardLayout::Korean);
    }

    #[test]
    fn korean_toggle_emits_ime_key_and_preserves_physical_qwerty_mapping() {
        let mut keyboard = KeyboardSession::default();
        assert_eq!(
            keyboard.press("layout").unwrap(),
            Some(KeyStroke {
                virtual_key: VK_HANGUL,
                shift: false
            })
        );
        assert_eq!(keyboard.layout(), KeyboardLayout::Korean);
        assert_eq!(
            keyboard.press("r").unwrap().unwrap().virtual_key,
            b'R' as u16
        );
    }

    #[test]
    fn tracking_loss_cancels_keyboard_tap_and_long_pinch_does_not_click() {
        let start = Instant::now();
        let mut tap = KeyboardTapRecognizer::default();
        assert!(!tap.update(Some(0.2), start));
        assert!(!tap.update(None, start + Duration::from_millis(50)));
        assert!(!tap.update(Some(0.2), start + Duration::from_millis(500)));
        assert!(!tap.update(Some(0.2), start + Duration::from_millis(1_250)));
        assert!(!tap.update(Some(0.5), start + Duration::from_millis(1_260)));
    }

    #[test]
    fn overlong_pinch_stays_blocked_until_release_instead_of_rearming_as_a_tap() {
        let start = Instant::now();
        let mut tap = KeyboardTapRecognizer::default();
        assert!(!tap.update(Some(0.2), start));
        assert!(!tap.update(Some(0.2), start + Duration::from_millis(710)));
        assert!(!tap.update(Some(0.2), start + Duration::from_millis(720)));
        assert!(!tap.update(Some(0.5), start + Duration::from_millis(730)));
    }

    #[test]
    fn hand_click_bounds_include_edges_but_exclude_points_outside_the_keyboard() {
        let bounds = KeyboardBounds {
            left: -200,
            top: 400,
            width: 800,
            height: 280,
        };
        assert!(bounds.contains([-200, 400]));
        assert!(bounds.contains([599, 679]));
        assert!(!bounds.contains([-201, 400]));
        assert!(!bounds.contains([600, 679]));
        assert!(!bounds.contains([0, 680]));
        assert!(!KeyboardBounds { width: 0, ..bounds }.contains([-200, 400]));
    }

    #[test]
    fn short_pinch_clicks_once_after_release_and_applies_cooldown() {
        let start = Instant::now();
        let mut tap = KeyboardTapRecognizer::default();
        assert!(!tap.update(Some(0.2), start));
        assert!(tap.update(Some(0.5), start + Duration::from_millis(80)));
        assert!(!tap.update(Some(0.2), start + Duration::from_millis(100)));
    }
}
