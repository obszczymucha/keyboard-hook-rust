use core::fmt;

pub struct Modifiers {
    // left_control: bool,
    // right_control: bool,
    // left_shift: bool,
    // right_shift: bool,
    // left_win: bool,
    // right_win: bool,
    pub left_alt: bool,
    // right_alt: bool,
}

pub enum Action {
    Hello,
    Bye,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Hello => write!(f, "Hello"),
            Action::Bye => write!(f, "Bye"),
        }
    }
}
