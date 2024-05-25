use std::sync::mpsc;

use crate::types::Action;

pub struct ActionHandler;

impl ActionHandler {
    pub fn consume(receiver: mpsc::Receiver<Action>) {
        for action in receiver {
            println!("Consumed: {}", action);
            match action {
                Action::Hello => println!("Keyboard hooked. Press Alt+A and then X to exit."),
                Action::Bye => println!("Bye bye!"),
                Action::ChannelToggles(toggles) => println!("Got toggles: {:?}", toggles),
            }
        }
    }
}
