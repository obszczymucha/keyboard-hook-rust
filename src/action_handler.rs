use crate::types::{ActionType, KeyPresses};
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
    pub fn consume(&self, receiver: mpsc::Receiver<ActionType>) {
        for action in receiver {
            match action {
                ActionType::Hello => {
                    println!("Keyboard hooked. Press Alt+A -> E -> X -> I -> T to exit.")
                }
                ActionType::Bye => println!("Exiting..."),
                ActionType::ToggleChannels => println!("ToggleChannels"),
                ActionType::Volume => println!("Volume"),
            }
        }
    }
}
