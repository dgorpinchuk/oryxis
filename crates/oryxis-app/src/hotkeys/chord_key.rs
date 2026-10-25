//! The key a chord is matched by, under any keyboard layout.
//!
//! iced hands a `KeyPressed` two readings of the same press: `key` is
//! winit's `key_without_modifiers`, the character the active layout
//! types on that key with nothing held, and `physical_key` is the key's
//! position. Every chord in the app (the hotkey table, the three
//! recorders, the widget's copy / select-all, the control byte the PTY
//! gets) used to read `key` alone. That is the layout's character on
//! EVERY platform, so under a Russian layout the key labelled C reads
//! `с`: Cmd+C, Ctrl+Shift+C and the Shortcuts capture all stopped at
//! the layout switch (PR #270), and Ctrl+C had no control byte to send.
//!
//! The rule here is the one Chrome, VS Code and Windows Terminal apply.
//! A chord is matched by the layout's character when that character is
//! ASCII (AZERTY's Ctrl+A is the key labelled A, Dvorak's Cmd+C the key
//! labelled C, as the OS itself has it), and otherwise by the character
//! the US layout puts on the same physical key. Only a chord is read
//! this way: with no Ctrl / Alt / Logo held the press is typing, and
//! the layout's character is the text.

use iced::keyboard::key::{Code, Physical};
use iced::keyboard::{Key, Modifiers};

/// The key a chord matcher reads for this press: `key` itself unless a
/// chord modifier is held, `key` is a character outside ASCII and the
/// physical key has a US-layout character, in which case that character
/// (unshifted, lowercase, the shape `key` has on a US layout).
///
/// A named key, a dead key and an unidentified key pass through: only a
/// character the layout wrote can stand in the way of a chord. So does a
/// position the US layout has no character for (a numpad key, the ISO
/// key beside the left Shift): nothing is invented for those.
pub fn chord_key(key: &Key, physical_key: &Physical, modifiers: &Modifiers) -> Key {
    if !(modifiers.control() || modifiers.alt() || modifiers.logo()) {
        return key.clone();
    }
    let Key::Character(typed) = key else {
        return key.clone();
    };
    if typed.is_ascii() {
        return key.clone();
    }
    let Physical::Code(code) = physical_key else {
        return key.clone();
    };
    match us_layout_character(*code) {
        Some(us) => Key::Character(us.into()),
        None => key.clone(),
    }
}

/// The unshifted character of the US layout at this position, for the
/// keys the binding grammar can hold: the letters, the digits and the
/// punctuation keys of the main block.
fn us_layout_character(code: Code) -> Option<&'static str> {
    Some(match code {
        Code::KeyA => "a",
        Code::KeyB => "b",
        Code::KeyC => "c",
        Code::KeyD => "d",
        Code::KeyE => "e",
        Code::KeyF => "f",
        Code::KeyG => "g",
        Code::KeyH => "h",
        Code::KeyI => "i",
        Code::KeyJ => "j",
        Code::KeyK => "k",
        Code::KeyL => "l",
        Code::KeyM => "m",
        Code::KeyN => "n",
        Code::KeyO => "o",
        Code::KeyP => "p",
        Code::KeyQ => "q",
        Code::KeyR => "r",
        Code::KeyS => "s",
        Code::KeyT => "t",
        Code::KeyU => "u",
        Code::KeyV => "v",
        Code::KeyW => "w",
        Code::KeyX => "x",
        Code::KeyY => "y",
        Code::KeyZ => "z",
        Code::Digit0 => "0",
        Code::Digit1 => "1",
        Code::Digit2 => "2",
        Code::Digit3 => "3",
        Code::Digit4 => "4",
        Code::Digit5 => "5",
        Code::Digit6 => "6",
        Code::Digit7 => "7",
        Code::Digit8 => "8",
        Code::Digit9 => "9",
        Code::Backquote => "`",
        Code::Minus => "-",
        Code::Equal => "=",
        Code::BracketLeft => "[",
        Code::BracketRight => "]",
        Code::Backslash => "\\",
        Code::Semicolon => ";",
        Code::Quote => "'",
        Code::Comma => ",",
        Code::Period => ".",
        Code::Slash => "/",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkeys::{binding_from_event, HotkeyBinding, PrimaryKey};
    use iced::keyboard::key::{Named, NativeCode};

    fn ch(s: &str) -> Key {
        Key::Character(s.into())
    }

    fn at(code: Code) -> Physical {
        Physical::Code(code)
    }

    const CTRL_SHIFT: Modifiers = Modifiers::CTRL.union(Modifiers::SHIFT);

    /// The report of PR #270: Russian PC on macOS, Cmd held. The key
    /// labelled C types `с`, V types `м`, A types `ф`; the chord is the
    /// letter printed on the key.
    #[test]
    fn a_non_latin_layout_matches_by_the_us_position() {
        for (typed, code, us) in [("с", Code::KeyC, "c"), ("м", Code::KeyV, "v"), ("ф", Code::KeyA, "a")] {
            assert_eq!(chord_key(&ch(typed), &at(code), &Modifiers::LOGO), ch(us));
            assert_eq!(chord_key(&ch(typed), &at(code), &Modifiers::CTRL), ch(us));
            assert_eq!(chord_key(&ch(typed), &at(code), &CTRL_SHIFT), ch(us));
            assert_eq!(chord_key(&ch(typed), &at(code), &Modifiers::ALT), ch(us));
        }
    }

    /// A letter of each script the app ships a UI language for, placed
    /// on a letter key: the rule reads only whether the character is
    /// ASCII, so which key a layout really puts it on does not matter
    /// here. The digit row is in the set too (the Persian layout types
    /// its own digits there).
    #[test]
    fn every_bundled_script_resolves_the_same_way() {
        for (typed, code, us) in [
            ("і", Code::KeyS, "s"),  // Ukrainian
            ("ψ", Code::KeyC, "c"),  // Greek
            ("ב", Code::KeyC, "c"),  // Hebrew
            ("ؤ", Code::KeyC, "c"),  // Arabic
            ("ژ", Code::KeyC, "c"),  // Persian
            ("แ", Code::KeyC, "c"),  // Thai
            ("म", Code::KeyC, "c"),  // Hindi
            ("۱", Code::Digit1, "1"), // a Persian digit
        ] {
            assert_eq!(chord_key(&ch(typed), &at(code), &Modifiers::CTRL), ch(us), "{typed}");
        }
    }

    /// The punctuation keys the binding grammar knows, with the Russian
    /// PC layout's characters on them: Ctrl+, opens Settings from the key
    /// that types `б`, and Ctrl+[ is the shell's Escape on the key that
    /// types `х`.
    #[test]
    fn the_punctuation_keys_resolve_to_the_us_glyph() {
        for (typed, code, us) in [
            ("б", Code::Comma, ","),
            ("ю", Code::Period, "."),
            ("ж", Code::Semicolon, ";"),
            ("э", Code::Quote, "'"),
            ("х", Code::BracketLeft, "["),
            ("ъ", Code::BracketRight, "]"),
            ("ё", Code::Backquote, "`"),
        ] {
            assert_eq!(chord_key(&ch(typed), &at(code), &Modifiers::CTRL), ch(us), "{typed}");
        }
    }

    /// A Latin layout keeps its own letters: AZERTY's Ctrl+A is the key
    /// labelled A (physically Q), German's Ctrl+Z the key labelled Z
    /// (physically Y), Dvorak's Cmd+C the key labelled C (physically I).
    /// The OS binds its own menu shortcuts that way and so does this.
    #[test]
    fn an_ascii_character_wins_over_the_position() {
        assert_eq!(chord_key(&ch("a"), &at(Code::KeyQ), &Modifiers::CTRL), ch("a"));
        assert_eq!(chord_key(&ch("z"), &at(Code::KeyY), &Modifiers::CTRL), ch("z"));
        assert_eq!(chord_key(&ch("c"), &at(Code::KeyI), &Modifiers::LOGO), ch("c"));
        assert_eq!(chord_key(&ch(","), &at(Code::KeyW), &Modifiers::CTRL), ch(","));
    }

    /// With no chord modifier the press is typing: `с` stays `с`, or a
    /// Russian user could not type. Shift alone is typing too.
    #[test]
    fn typing_is_never_remapped() {
        assert_eq!(chord_key(&ch("с"), &at(Code::KeyC), &Modifiers::empty()), ch("с"));
        assert_eq!(chord_key(&ch("с"), &at(Code::KeyC), &Modifiers::SHIFT), ch("с"));
    }

    /// Nothing is invented: a named key, an unidentified key, an
    /// unidentified position and a position the US layout leaves blank
    /// pass through.
    #[test]
    fn keys_without_a_us_character_pass_through() {
        let named = Key::Named(Named::Enter);
        assert_eq!(chord_key(&named, &at(Code::Enter), &Modifiers::CTRL), named);
        let unknown = Key::Unidentified;
        assert_eq!(chord_key(&unknown, &at(Code::Equal), &Modifiers::CTRL), unknown);
        let unidentified = Physical::Unidentified(NativeCode::Unidentified);
        assert_eq!(chord_key(&ch("с"), &unidentified, &Modifiers::CTRL), ch("с"));
        assert_eq!(chord_key(&ch("с"), &at(Code::IntlBackslash), &Modifiers::CTRL), ch("с"));
        assert_eq!(chord_key(&ch("٣"), &at(Code::Numpad3), &Modifiers::CTRL), ch("٣"));
    }

    /// The recorder and the matcher read the resolved key, so a chord
    /// recorded under Russian is stored as the Latin letter and fires
    /// under English, and the other way round.
    #[test]
    fn a_chord_recorded_under_one_layout_fires_under_the_other() {
        let recorded = chord_key(&ch("с"), &at(Code::KeyC), &CTRL_SHIFT);
        let binding = binding_from_event(&recorded, &CTRL_SHIFT, true).expect("a letter chord records");
        assert_eq!(binding, HotkeyBinding { ctrl: true, shift: true, alt: false, logo: false, primary: PrimaryKey::Char('c') });
        assert!(binding.match_event(&ch("c"), &CTRL_SHIFT).is_some(), "fires under English");
        let under_russian = chord_key(&ch("с"), &at(Code::KeyC), &CTRL_SHIFT);
        assert!(binding.match_event(&under_russian, &CTRL_SHIFT).is_some(), "fires under Russian");
        assert!(binding.match_event(&ch("с"), &CTRL_SHIFT).is_none(), "the raw key never matched");
    }
}
