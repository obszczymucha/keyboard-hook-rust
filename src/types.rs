use crate::types::Modifier::*;
use core::fmt;
use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Modifier {
    NoMod,
    ModAlt,
}

impl Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::NoMod => write!(f, ""),
            Modifier::ModAlt => write!(f, "A"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(dead_code)]
pub enum Action {
    Hello,
    Bye,
    PrincessKenny,
    ChannelToggles(KeyPresses),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct KeyPresses(pub Vec<KeyPress>);

impl Display for KeyPresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        for key_press in &self.0 {
            result.push_str(&format!("{}", key_press));
        }
        write!(f, "{}", result)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Hello => write!(f, "Hello"),
            Action::Bye => write!(f, "Bye"),
            Action::PrincessKenny => write!(f, "PrincessKenny"),
            Action::ChannelToggles(toggles) => write!(f, "Toggles: {}", toggles),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Key {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    KeyA,
    KeyX,
    Unmapped(u8),
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key1 => write!(f, "1"),
            Key2 => write!(f, "2"),
            Key3 => write!(f, "3"),
            Key4 => write!(f, "4"),
            Key5 => write!(f, "5"),
            KeyA => write!(f, "A"),
            KeyX => write!(f, "X"),
            Unmapped(key) => write!(f, "Unmapped({})", key),
        }
    }
}

impl Key {
    pub fn from_u8(key: u8) -> Key {
        match key {
            b'1' => Key1,
            b'2' => Key2,
            b'3' => Key3,
            b'4' => Key4,
            b'5' => Key5,
            b'A' => KeyA,
            b'X' => KeyX,
            _ => Unmapped(key),
        }
    }
}

use Key::*;

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct KeyPress(Key, Modifier);

impl KeyPress {
    pub fn new(key: Key, modifier: Modifier) -> Self {
        Self(key, modifier)
    }
}

impl Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.1 {
            NoMod => write!(f, "{}", self.0),
            ModAlt => write!(f, "<{}-{}>", self.1, self.0),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(dead_code)]
pub enum Mapping {
    Timeout(KeyPress),
    Action(KeyPress, Action),
    ActionBeforeTimeout(KeyPress, Action),
    ActionAfterTimeout(KeyPresses, Action),
}

impl Mapping {
    pub fn matches_key(&self, key_press: &KeyPress) -> bool {
        match self {
            Timeout(key) => key == key_press,
            Action(key, _) => key == key_press,
            ActionBeforeTimeout(key, _) => key == key_press,
            ActionAfterTimeout(key_presses, _) => key_presses.0.contains(key_press),
        }
    }
}

impl Display for Mapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timeout(key) => write!(f, "TimeoutKey: {}", key),
            Action(key, action) => write!(f, "ActionKey: {} -> {}", key, action),
            ActionBeforeTimeout(key_press, action) => {
                write!(f, "ActionBeforeTimeoutKey: {} -> {}", key_press, action)
            }
            ActionAfterTimeout(key_presses, action) => {
                write!(f, "ActionAfterTimeoutKey: {} -> {}", key_presses, action)
            }
        }
    }
}

use Mapping::*;

pub const KEY_1: KeyPress = KeyPress(Key1, NoMod);
pub const KEY_2: KeyPress = KeyPress(Key2, NoMod);
pub const KEY_3: KeyPress = KeyPress(Key3, NoMod);
pub const KEY_4: KeyPress = KeyPress(Key4, NoMod);
pub const KEY_5: KeyPress = KeyPress(Key5, NoMod);
pub const KEY_A: KeyPress = KeyPress(KeyA, NoMod);
pub const KEY_X: KeyPress = KeyPress(KeyX, NoMod);
pub const ALT_A: KeyPress = KeyPress(KeyA, ModAlt);
