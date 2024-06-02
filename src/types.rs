use crate::types::Modifier::*;
use core::fmt;
use std::fmt::Debug;
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
pub enum ActionType<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    System(SystemActionType),
    User(T),
}

impl<T> Display for ActionType<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::System(action) => write!(f, "{}", action),
            ActionType::User(action) => write!(f, "{}", action),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Action<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    System(SystemActionType),
    User(T, Vec<KeyPress>),
}

impl<T> Display for Action<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::System(action) => write!(f, "{}", action),
            Action::User(action, _) => write!(f, "{}", action),
        }
    }
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

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
#[allow(dead_code)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Unmapped(u8),
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key0 => write!(f, "0"),
            Key1 => write!(f, "1"),
            Key2 => write!(f, "2"),
            Key3 => write!(f, "3"),
            Key4 => write!(f, "4"),
            Key5 => write!(f, "5"),
            Key6 => write!(f, "5"),
            Key7 => write!(f, "5"),
            Key8 => write!(f, "5"),
            Key9 => write!(f, "5"),
            KeyA => write!(f, "A"),
            KeyB => write!(f, "B"),
            KeyC => write!(f, "C"),
            KeyD => write!(f, "D"),
            KeyE => write!(f, "E"),
            KeyF => write!(f, "F"),
            KeyG => write!(f, "G"),
            KeyH => write!(f, "H"),
            KeyI => write!(f, "I"),
            KeyJ => write!(f, "J"),
            KeyK => write!(f, "K"),
            KeyL => write!(f, "L"),
            KeyM => write!(f, "M"),
            KeyN => write!(f, "N"),
            KeyO => write!(f, "O"),
            KeyP => write!(f, "P"),
            KeyQ => write!(f, "Q"),
            KeyR => write!(f, "R"),
            KeyS => write!(f, "S"),
            KeyT => write!(f, "T"),
            KeyU => write!(f, "U"),
            KeyV => write!(f, "V"),
            KeyW => write!(f, "W"),
            KeyX => write!(f, "X"),
            KeyY => write!(f, "Y"),
            KeyZ => write!(f, "Z"),
            Unmapped(key) => write!(f, "Unmapped({})", key),
        }
    }
}

impl Key {
    pub fn from_u8(key: u8) -> Key {
        match key {
            b'0' => Key0,
            b'1' => Key1,
            b'2' => Key2,
            b'3' => Key3,
            b'4' => Key4,
            b'5' => Key5,
            b'A' => KeyA,
            b'B' => KeyB,
            b'C' => KeyC,
            b'D' => KeyD,
            b'E' => KeyE,
            b'F' => KeyF,
            b'G' => KeyG,
            b'H' => KeyH,
            b'I' => KeyI,
            b'J' => KeyJ,
            b'K' => KeyK,
            b'L' => KeyL,
            b'M' => KeyM,
            b'N' => KeyN,
            b'O' => KeyO,
            b'P' => KeyP,
            b'Q' => KeyQ,
            b'R' => KeyR,
            b'S' => KeyS,
            b'T' => KeyT,
            b'U' => KeyU,
            b'V' => KeyV,
            b'W' => KeyW,
            b'X' => KeyX,
            b'Y' => KeyY,
            b'Z' => KeyZ,
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Mapping<T>
where
    T: Clone + PartialEq + Eq + Debug + Display + Sync + Send,
{
    Timeout(KeyPressType),
    Action(KeyPressType, ActionMapping<T>),
}

use Mapping::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ActionMapping<T>
where
    T: Clone + PartialEq + Eq + Debug + Display + Sync + Send,
{
    NoTimeout(ActionType<T>),
    TimeoutAfterAction(ActionType<T>),
    TimeoutBeforeAction(ActionType<T>),
}

use ActionMapping::*;

impl<T> Mapping<T>
where
    T: Clone + PartialEq + Eq + Debug + Display + Sync + Send,
{
    pub fn get_key(&self) -> &KeyPressType {
        match self {
            Timeout(key) => key,
            Action(key, action_mapping) => match action_mapping {
                NoTimeout(_) => key,
                TimeoutAfterAction(_) => key,
                TimeoutBeforeAction(_) => key,
            },
        }
    }
}

impl<T> Display for Mapping<T>
where
    T: Clone + PartialEq + Eq + Debug + Display + Sync + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timeout(key) => write!(f, "Timeout: {}", key),
            Action(key, action_mapping) => match action_mapping {
                NoTimeout(action_type) => write!(f, "Action: {} -> {}", key, action_type),
                TimeoutAfterAction(action_type) => {
                    write!(f, "TimeoutAfterAction: {} -> {}", key, action_type)
                }
                TimeoutBeforeAction(action) => {
                    write!(f, "ActionAfterTimeoutKey: {} -> {}", key, action)
                }
            },
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SystemActionType {
    Hello,
    Bye,
}

impl Display for SystemActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemActionType::Hello => write!(f, "Hello"),
            SystemActionType::Bye => write!(f, "Bye"),
        }
    }
}
