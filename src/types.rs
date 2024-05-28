use core::fmt;
use std::fmt::Display;

use crate::mapping_handler::KeyPress;

#[derive(PartialEq, Eq, Clone, Hash)]
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

#[derive(PartialEq, Eq, Clone)]
#[allow(dead_code)]
pub enum Action {
    Hello,
    Bye,
    PrincessKenny,
    ChannelToggles(KeyPresses),
}

#[derive(PartialEq, Eq, Clone)]
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
