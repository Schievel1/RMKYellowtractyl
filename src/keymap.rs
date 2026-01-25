use embassy_time::Duration;
use rmk::combo::Combo;
use rmk::combo::ComboConfig;
use rmk::config::macro_config::KeyboardMacrosConfig;
use rmk::config::CombosConfig;
use rmk::keyboard_macros::define_macro_sequences;
use rmk::keyboard_macros::to_macro_sequence;
use rmk::types::action::Action;
use rmk::types::action::{KeyAction, MorseProfile};
use rmk::types::keycode::{HidKeyCode, KeyCode};
use rmk::types::modifier::ModifierCombination;
use rmk::{a, k, layer, mo, shifted, wm};

pub(crate) const COL: usize = 12;
pub(crate) const ROW: usize = 6;
pub(crate) const NUM_LAYER: usize = 3;

const LCTRL: ModifierCombination = ModifierCombination::LCTRL;
// const LSHIFT: ModifierCombination = ModifierCombination::LSHIFT;
// const RSHIFT: ModifierCombination = ModifierCombination::RSHIFT;

const SC_LSHIFT: KeyAction = KeyAction::TapHold(
    Action::TriggerMacro(0),
    Action::Key(KeyCode::Hid(HidKeyCode::LShift)),
    MorseProfile::const_default(),
);
const SC_RSHIFT: KeyAction = KeyAction::TapHold(
    Action::TriggerMacro(1),
    Action::Key(KeyCode::Hid(HidKeyCode::RShift)),
    MorseProfile::const_default(),
);
const USER0: KeyAction = KeyAction::Single(Action::User(0));
#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        layer!([
[k!(Grave),  k!(Kc1),     k!(Kc2),     k!(Kc3),       k!(Kc4),    k!(Kc5),                     k!(Kc6),     k!(Kc7),     k!(Kc8),     k!(Kc9),     k!(Kc0),        k!(Equal)],
[k!(Tab),    k!(Quote),   k!(Comma),   k!(Dot),       k!(P),      k!(Y),                       k!(F),       k!(G),       k!(C),       k!(R),       k!(L),        k!(Slash)],
[k!(Escape), k!(A),       k!(O),       k!(E),       k!(U),      k!(I),                         k!(D),       k!(H),       k!(T),       k!(N),       k!(S),        k!(Minus)],
[a!(No),   k!(Semicolon), k!(Q),       k!(J),       k!(K),      k!(X),                         k!(B),       k!(M),       k!(W),       k!(V),       k!(Z),        k!(Backslash)],
            [a!(No), a!(No),           k!(LeftBracket), k!(RightBracket), SC_LSHIFT, k!(Space), a!(No), SC_RSHIFT, k!(PageUp), k!(PageDown), a!(No), a!(No)],
[a!(No), a!(No), k!(LAlt),     k!(LGui),  k!(LCtrl),    mo!(1),   a!(No),                     k!(Backspace), k!(Enter),  k!(RGui), a!(No), a!(No)]
        ]),
        layer!([
[a!(No),      k!(F1),       k!(F2),      k!(F3),      k!(F4),     k!(F5),                        k!(F6),        k!(F7),       k!(F8),      k!(F9),      k!(F10),        k!(Delete)],
[a!(No),      a!(No),       a!(No),      a!(No),      a!(No), shifted!(LeftBracket),    shifted!(RightBracket), k!(MouseBtn2), a!(No),   a!(No),       a!(No),        a!(No)],
[USER0,   a!(No),       a!(No),      mo!(2),      k!(Delete), shifted!(Kc9),           shifted!(Kc0), k!(Left),    k!(Up),      k!(Down),     k!(Right),    a!(No)],
[k!(CapsLock), a!(No),      a!(No),     wm!(X, LCTRL), wm!(C, LCTRL), wm!(V, LCTRL),             a!(No),         k!(MouseBtn1), a!(No),      a!(No),       a!(No),        a!(No)],
[a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No),                                                              a!(No), a!(No)],
[a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No),                                                              a!(No), a!(No)]
        ]),
        layer!([
[a!(No),      a!(No),       a!(No),      a!(No),       a!(No),     a!(No),                        a!(No),       k!(NumLock), k!(KpSlash), k!(KpAsterisk), k!(KpMinus), k!(Calculator)],
[a!(No),      a!(No),       a!(No),      a!(No),       a!(No),     a!(No),                        a!(No),        k!(Kp7),    k!(Kp8),    k!(Kp9),    k!(KpPlus),  k!(AudioMute)],
[a!(No),      a!(No),       a!(No),      a!(No),       a!(No),     a!(No),                        a!(No),        k!(Kp4),    k!(Kp5),    k!(Kp6),    a!(No),      k!(AudioVolUp)],
[a!(No),      a!(No),       a!(No),      a!(No),       a!(No),     a!(No),                        k!(Kp0),       k!(Kp1),    k!(Kp2),    k!(Kp3),    k!(KpEqual), k!(AudioVolDown)],
[a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No),                                             k!(KpDot),   k!(KpComma)],
[a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No), a!(No)]
        ]),
    ]
}
pub fn get_macros() -> KeyboardMacrosConfig {
    KeyboardMacrosConfig::new(define_macro_sequences(&[
        to_macro_sequence("{"),
        to_macro_sequence("}"),
    ]))
}

pub fn get_combos() -> CombosConfig {
    CombosConfig {
        timeout: Duration::from_millis(50),
        combos: [
            Some(Combo::new(ComboConfig::new(
                [k!(H), k!(T)],
                k!(Escape),
                Some(0),
            ))),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    }
}
