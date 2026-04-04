//! Keyboard event types and key definitions.
//!
//! Provides an abstraction over raw windowing keyboard events. The [`Key`] enum
//! maps to a subset of `winit::keyboard::Key` / `NamedKey`, and
//! [`KeyboardEvent`] wraps press / release / IME events.

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key as WinitKey, NamedKey};

// ─────────────────────────────────────────────────────────────────────
// Key
// ─────────────────────────────────────────────────────────────────────

/// Logical key identifier (corresponds to `winit::keyboard::Key`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// A printable character (one or more Unicode scalars).
    Character(String),
    /// The Enter / Return key.
    Enter,
    /// The Tab key.
    Tab,
    /// The Backspace key.
    Backspace,
    /// The Delete key.
    Delete,
    /// The left arrow key.
    Left,
    /// The right arrow key.
    Right,
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
    /// The Home key.
    Home,
    /// The End key.
    End,
    /// The Escape key.
    Escape,
    /// The space bar.
    Space,
    /// A named key that we have not mapped to a dedicated variant.
    Other(String),
}

impl Key {
    /// Convert from a `winit::keyboard::Key` reference.
    pub fn from_winit(winit_key: &WinitKey) -> Self {
        match winit_key {
            WinitKey::Named(named) => Self::from_named(*named),
            WinitKey::Character(s) => Key::Character(s.to_string()),
            WinitKey::Unidentified(_) => Key::Other("Unidentified".into()),
            WinitKey::Dead(ch) => Key::Other(format!("Dead({:?})", ch)),
        }
    }

    fn from_named(named: NamedKey) -> Self {
        match named {
            NamedKey::Enter => Key::Enter,
            NamedKey::Tab => Key::Tab,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Delete => Key::Delete,
            NamedKey::ArrowLeft => Key::Left,
            NamedKey::ArrowRight => Key::Right,
            NamedKey::ArrowUp => Key::Up,
            NamedKey::ArrowDown => Key::Down,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::Escape => Key::Escape,
            NamedKey::Space => Key::Space,
            other => Key::Other(format!("{:?}", other)),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Modifiers
// ─────────────────────────────────────────────────────────────────────

/// Modifier key state snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Whether Shift is currently pressed.
    pub shift: bool,
    /// Whether Control is currently pressed.
    pub ctrl: bool,
    /// Whether Alt/Option is currently pressed.
    pub alt: bool,
    /// Cmd on macOS, Win key on Windows.
    pub meta: bool,
}

// ─────────────────────────────────────────────────────────────────────
// KeyboardEvent
// ─────────────────────────────────────────────────────────────────────

/// High-level keyboard / IME events.
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// A key was pressed.
    KeyDown {
        /// Logical key value.
        key: Key,
        /// Modifier snapshot captured for this event.
        modifiers: Modifiers,
        /// The text produced by this key press, if any.
        text: Option<String>,
    },
    /// A key was released.
    KeyUp {
        /// Logical key value.
        key: Key,
        /// Modifier snapshot captured for this event.
        modifiers: Modifiers,
    },
    /// IME composition committed.
    ImeCommit(String),
    /// IME pre-edit (composing) state changed.
    /// The tuple is `(cursor_start, cursor_end)` if available.
    ImePreedit(String, Option<(usize, usize)>),
}

// ─────────────────────────────────────────────────────────────────────
// Conversion from winit::event::KeyEvent
// ─────────────────────────────────────────────────────────────────────

impl KeyboardEvent {
    /// Create a `KeyboardEvent` from a winit `KeyEvent` and the current
    /// modifier state.
    ///
    /// The caller must supply `modifiers` separately because winit tracks
    /// modifier state on the window, not on individual key events.
    pub fn from_winit(event: &KeyEvent, modifiers: Modifiers) -> Self {
        let key = Key::from_winit(&event.logical_key);
        match event.state {
            ElementState::Pressed => KeyboardEvent::KeyDown {
                key,
                modifiers,
                text: event.text.as_ref().map(|s| s.to_string()),
            },
            ElementState::Released => KeyboardEvent::KeyUp { key, modifiers },
        }
    }
}

impl From<&KeyEvent> for KeyboardEvent {
    /// Convert a winit `KeyEvent` into a `KeyboardEvent`.
    ///
    /// **Note:** winit `KeyEvent` does not carry modifier state — modifiers
    /// default to all-false.  Use [`KeyboardEvent::from_winit`] when you have
    /// the modifier snapshot available.
    fn from(event: &KeyEvent) -> Self {
        Self::from_winit(event, Modifiers::default())
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_from_named_enter() {
        assert_eq!(Key::from_named(NamedKey::Enter), Key::Enter);
    }

    #[test]
    fn key_from_named_arrows() {
        assert_eq!(Key::from_named(NamedKey::ArrowLeft), Key::Left);
        assert_eq!(Key::from_named(NamedKey::ArrowRight), Key::Right);
        assert_eq!(Key::from_named(NamedKey::ArrowUp), Key::Up);
        assert_eq!(Key::from_named(NamedKey::ArrowDown), Key::Down);
    }

    #[test]
    fn key_from_named_special() {
        assert_eq!(Key::from_named(NamedKey::Tab), Key::Tab);
        assert_eq!(Key::from_named(NamedKey::Backspace), Key::Backspace);
        assert_eq!(Key::from_named(NamedKey::Delete), Key::Delete);
        assert_eq!(Key::from_named(NamedKey::Home), Key::Home);
        assert_eq!(Key::from_named(NamedKey::End), Key::End);
        assert_eq!(Key::from_named(NamedKey::Escape), Key::Escape);
        assert_eq!(Key::from_named(NamedKey::Space), Key::Space);
    }

    #[test]
    fn key_from_named_unmapped_goes_to_other() {
        let k = Key::from_named(NamedKey::F1);
        assert!(matches!(k, Key::Other(_)));
    }

    #[test]
    fn key_from_winit_character() {
        let wk = WinitKey::Character("a".into());
        assert_eq!(Key::from_winit(&wk), Key::Character("a".into()));
    }

    #[test]
    fn key_from_winit_named() {
        let wk = WinitKey::Named(NamedKey::Escape);
        assert_eq!(Key::from_winit(&wk), Key::Escape);
    }

    #[test]
    fn modifiers_default_all_false() {
        let m = Modifiers::default();
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn keyboard_event_variants_are_clone() {
        let ev = KeyboardEvent::KeyDown {
            key: Key::Enter,
            modifiers: Modifiers::default(),
            text: Some("\r".into()),
        };
        let _cloned = ev.clone();
    }

    #[test]
    fn ime_commit_event() {
        let ev = KeyboardEvent::ImeCommit("日本語".into());
        if let KeyboardEvent::ImeCommit(s) = ev {
            assert_eq!(s, "日本語");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn ime_preedit_event() {
        let ev = KeyboardEvent::ImePreedit("にほん".into(), Some((0, 3)));
        if let KeyboardEvent::ImePreedit(s, cursor) = ev {
            assert_eq!(s, "にほん");
            assert_eq!(cursor, Some((0, 3)));
        } else {
            panic!("wrong variant");
        }
    }
}
