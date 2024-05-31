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
    ToggleChannels,
    Volume,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct KeyPresses(pub Vec<KeyPress>);

impl KeyPresses {
    pub fn choice(&self) -> KeyPressType {
        KeyPressType::Choice(self.clone())
    }
}

impl Display for KeyPresses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self
            .0
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        write!(f, "{}", result)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Hello => write!(f, "Hello"),
            Action::Bye => write!(f, "Bye"),
            Action::ToggleChannels => write!(f, "ToggleChannels"),
            Action::Volume => write!(f, "Volume"),
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
    KeyE,
    KeyI,
    KeyJ,
    KeyK,
    KeyT,
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
            KeyE => write!(f, "E"),
            KeyI => write!(f, "I"),
            KeyJ => write!(f, "J"),
            KeyK => write!(f, "K"),
            KeyT => write!(f, "T"),
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
            b'E' => KeyE,
            b'I' => KeyI,
            b'J' => KeyJ,
            b'K' => KeyK,
            b'T' => KeyT,
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

    pub fn nomod(key: Key) -> Self {
        Self(key, NoMod)
    }

    #[allow(unused)]
    pub fn alt(key: Key) -> Self {
        Self(key, ModAlt)
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

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum KeyPressType {
    Single(KeyPress),
    Choice(KeyPresses),
}

impl Display for KeyPressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Single(key) => write!(f, "{}", key),
            Choice(keys) => write!(f, "{}", keys),
        }
    }
}
use KeyPressType::*;

#[derive(PartialEq, Eq, Debug, Clone)]
#[allow(dead_code)]
pub enum Mapping {
    Timeout(KeyPressType),
    Action(KeyPressType, Action),
    ActionBeforeTimeout(KeyPressType, Action),
    ActionAfterTimeout(KeyPressType, Action),
}

use Mapping::*;

impl Mapping {
    pub fn get_key(&self) -> &KeyPressType {
        match self {
            Timeout(key) => key,
            Action(key, _) => key,
            ActionBeforeTimeout(key, _) => key,
            ActionAfterTimeout(key, _) => key,
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
