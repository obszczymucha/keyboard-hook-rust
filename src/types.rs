use core::fmt;

#[derive(PartialEq, Eq)]
pub enum Modifier {
    Alt,
}

pub enum Action {
    Hello,
    Bye,
    ChannelToggles(Vec<u32>),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Hello => write!(f, "Hello"),
            Action::Bye => write!(f, "Bye"),
            Action::ChannelToggles(toggles) => write!(f, "Toggles: {:?}", toggles),
        }
    }
}
