use crate::types::Modifier::*;
use core::fmt;
use core::hash::Hash;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Modifier {
    NoMod, // TODO: Remove this.
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

/// This is an event that gets emitted by the keyboard_hook.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Event<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    System(SystemAction),
    Single(A),
    Multi(T, Vec<A>),
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
pub enum KeyPress {
    // TODO: I think it makes more sense to remove 'NoMod' modifier from Modifier.
    // NoMod(Key),
    Mod(Key, Modifier),
}

impl Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // KeyPress::NoMod(key) => write!(f, "{}", key),
            KeyPress::Mod(key, NoMod) => write!(f, "{}", key),
            KeyPress::Mod(key, modifier) => write!(f, "<{}-{}>", modifier, key),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum Behaviour<A>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    Timeout(KeyPress),
    Action(KeyPress, A),
    ActionOnTimeout(KeyPress, A),
}

impl<A> Behaviour<A>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    pub fn new(key: Key, modifier: Modifier) -> Self {
        Behaviour::Timeout(KeyPress::Mod(key, modifier))
    }

    pub fn a(key: Key, modifier: Modifier, action: A) -> Self {
        Behaviour::Action(KeyPress::Mod(key, modifier), action)
    }

    pub fn nomod(key: Key) -> Self {
        Behaviour::Timeout(KeyPress::Mod(key, NoMod))
    }

    pub fn nomod_a(key: Key, action: A) -> Self {
        Behaviour::Action(KeyPress::Mod(key, NoMod), action)
    }

    #[allow(unused)]
    pub fn alt(key: Key) -> Self {
        Behaviour::Timeout(KeyPress::Mod(key, ModAlt))
    }

    #[allow(unused)]
    pub fn alt_a(key: Key, action: A) -> Self {
        Behaviour::Action(KeyPress::Mod(key, ModAlt), action)
    }

    // TODO: Return the reference
    pub fn get_key(&self) -> KeyPress {
        match self {
            Behaviour::Timeout(key) => key.clone(),
            Behaviour::Action(key, _) => key.clone(),
            Behaviour::ActionOnTimeout(key, _) => key.clone(),
        }
    }

    pub fn get_modifier(&self) -> &Modifier {
        match self {
            Behaviour::Timeout(KeyPress::Mod(_, modifier)) => modifier,
            Behaviour::Action(KeyPress::Mod(_, modifier), _) => modifier,
            Behaviour::ActionOnTimeout(KeyPress::Mod(_, modifier), _) => modifier,
        }
    }
}

impl<A> Display for Behaviour<A>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Behaviour::Timeout(key) => write!(f, "NoActionMapping: {}", key),
            Behaviour::Action(key, action_type) => {
                write!(f, "ActionMapping: {} -> {}", key, action_type)
            }
            Behaviour::ActionOnTimeout(key, action_type) => {
                write!(f, "ActionOnTimeoutMapping: {} -> {}", key, action_type)
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Behaviours<A>(pub Vec<Behaviour<A>>)
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send;

impl<A> Behaviours<A>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    pub fn get_mapping(&self, key: &KeyPress) -> Option<&Behaviour<A>> {
        self.0.iter().find(|mapping| mapping.get_key() == *key)
    }

    // pub fn get_action_type(&self, key: KeyPress) -> Option<&Action<A, T>> {
    //     for key_mapping in self.0 {
    //         match key_mapping {
    //             KeyPressMapping::Action(k, a) if k == key => Some(a),
    //             _ => continue,
    //         };
    //     }
    //
    //     None
    // }
}

impl<A> Display for Behaviours<A>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
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

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Mapping<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    Single(Behaviour<A>),
    Choice(Behaviours<A>, T),
}

impl<A, T> Display for Mapping<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Single(key) => write!(f, "{}", key),
            Choice(keys, tag) => write!(f, "({}): {}", tag, keys),
        }
    }
}
use Mapping::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum SystemAction {
    Hello,
    Bye,
}

impl Display for SystemAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemAction::Hello => write!(f, "Hello"),
            SystemAction::Bye => write!(f, "Bye"),
        }
    }
}

pub struct ShutdownAction;
