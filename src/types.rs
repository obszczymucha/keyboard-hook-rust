use core::fmt;

pub struct Modifiers {
    pub left_alt: bool,
}

pub enum Action {
    Hello,
    Bye,
    ChannelToggles(Vec<u32>),
}

fn deduplicate(numbers: &Vec<u32>) -> Vec<u32> {
    let mut deduplicated = vec![];
    let mut occurrences = std::collections::HashMap::new();

    for number in numbers {
        let count = occurrences.entry(number).or_insert(0);
        *count += 1;
    }

    for (number, count) in occurrences {
        if count % 2 == 1 {
            deduplicated.push(*number);
        }
    }

    deduplicated
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Hello => write!(f, "Hello"),
            Action::Bye => write!(f, "Bye"),
            Action::ChannelToggles(toggles) => write!(f, "Toggles: {:?}", deduplicate(toggles)),
        }
    }
}
