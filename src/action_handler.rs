use crate::types::{Action, KeyPresses};
use std::{collections::HashSet, sync::mpsc};

pub struct ActionHandler;

#[allow(dead_code)]
fn deduplicate(key_presses: &KeyPresses) -> KeyPresses {
    let mut result = HashSet::new();

    for key_press in key_presses.0.iter() {
        if result.contains(key_press) {
            result.remove(key_press);
        } else {
            result.insert(key_press.clone());
        }
    }

    KeyPresses(result.into_iter().collect())
}

impl ActionHandler {
    pub fn consume(receiver: mpsc::Receiver<Action>) {
        for action in receiver {
            match action {
                Action::Hello => {
                    println!("Keyboard hooked. Press Alt+A -> E -> X -> I -> T to exit.")
                }
                Action::Bye => println!("Exiting..."),
                Action::ToggleChannels => println!("ToggleChannels"),
                Action::Volume => println!("Volume"),
            }
        }
    }
}
