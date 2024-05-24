use std::sync::mpsc;

use crate::types::Action;

pub struct ActionHandler {
    receiver: mpsc::Receiver<Action>,
}

impl ActionHandler {
    pub fn new(receiver: mpsc::Receiver<Action>) -> ActionHandler {
        ActionHandler { receiver }
    }

    pub fn start(&self) {
        for action in &self.receiver {
            println!("Consumed: {}", action);
            match action {
                Action::Hello => {
                    println!("Keyboard hooked. Press Alt+A and then X to exit.");
                }
                Action::Bye => {
                    println!("Bye bye!");
                }
            }
        }
    }
}
