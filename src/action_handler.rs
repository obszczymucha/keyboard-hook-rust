use crate::types::Action;
use std::{collections::HashSet, sync::mpsc};

pub struct ActionHandler;

fn deduplicate(numbers: &Vec<u32>) -> Vec<u32> {
    let mut result = HashSet::new();

    for number in numbers {
        if result.contains(number) {
            result.remove(number);
        } else {
            result.insert(*number);
        }
    }

    result.into_iter().collect()
}

impl ActionHandler {
    pub fn consume(receiver: mpsc::Receiver<Action>) {
        for action in receiver {
            println!("Consumed: {}", action);
            match action {
                Action::Hello => println!("Keyboard hooked. Press Alt+A and then X to exit."),
                Action::Bye => println!("Captured sequence: Alt+A -> X. Exiting..."),
                Action::ChannelToggles(toggles) => {
                    println!("Got toggles: {:?}", deduplicate(&toggles))
                }
            }
        }
    }
}
