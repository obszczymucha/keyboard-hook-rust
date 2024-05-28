use crate::types::Action::*;
use crate::types::KeyPresses;
use crate::types::Modifier::*;
use crate::types::{Action, Modifier};
use std::fmt::Display;

#[derive(PartialEq, Eq, Clone, Hash)]
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

#[derive(PartialEq, Eq, Clone, Hash)]
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

#[derive(PartialEq, Eq)]
#[allow(dead_code)]
pub enum Mapping {
    Timeout(KeyPress),
    Action(KeyPress, Action),
    ActionBeforeTimeout(KeyPress, Action),
    ActionAfterTimeout(KeyPresses, Action),
}

impl Mapping {
    fn matches_key(&self, key_press: &KeyPress) -> bool {
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

const KEY_1: KeyPress = KeyPress(Key1, NoMod);
const KEY_2: KeyPress = KeyPress(Key2, NoMod);
const KEY_3: KeyPress = KeyPress(Key3, NoMod);
const KEY_4: KeyPress = KeyPress(Key4, NoMod);
const KEY_5: KeyPress = KeyPress(Key5, NoMod);
const KEY_A: KeyPress = KeyPress(KeyA, NoMod);
const KEY_X: KeyPress = KeyPress(KeyX, NoMod);
const ALT_A: KeyPress = KeyPress(KeyA, ModAlt);

pub fn define_mappings() -> Vec<Vec<Mapping>> {
    vec![
        vec![Timeout(ALT_A), Action(KEY_X, Bye)],
        vec![Timeout(ALT_A), Timeout(KEY_A), Action(KEY_X, Bye)],
        vec![Timeout(ALT_A), Timeout(KEY_A), Action(KEY_A, PrincessKenny)],
        vec![
            Timeout(ALT_A),
            ActionAfterTimeout(
                KeyPresses(vec![KEY_1, KEY_2, KEY_3, KEY_4, KEY_5]),
                PrincessKenny,
            ),
        ],
    ]
}

pub struct MappingHandler {
    mappings: Vec<Vec<Mapping>>,
}

impl MappingHandler {
    pub fn new(mappings: Vec<Vec<Mapping>>) -> Self {
        Self { mappings }
    }

    pub fn handle_key_press(&self, buffer: &[KeyPress], key_press: &KeyPress) -> Option<&Mapping> {
        'mapping_loop: for mapping in &self.mappings {
            if mapping.len() <= buffer.len() {
                continue 'mapping_loop;
            }

            for (i, key_press_in_buffer) in buffer.iter().enumerate() {
                match &mapping[i] {
                    ActionAfterTimeout(key_presses, _) if i == buffer.len() - 1 => {
                        if key_presses.0.len() <= buffer.len()
                            && key_presses.0.iter().zip(buffer).all(|(kp, bp)| kp == bp)
                        {
                            return Some(&mapping[i]);
                        }
                    }
                    _ => {
                        if !mapping[i].matches_key(key_press_in_buffer) {
                            continue 'mapping_loop;
                        }
                    }
                }
            }

            let next_mapping = mapping.get(buffer.len());

            if let Some(next_mapping) = next_mapping {
                if next_mapping.matches_key(key_press) {
                    return Some(next_mapping);
                }
            }
        }

        None
    }
}
