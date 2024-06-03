use core::fmt;
use keyboard_hook::a;
use keyboard_hook::aat;
use keyboard_hook::abt;
use keyboard_hook::key;
use keyboard_hook::t;
use keyboard_hook::types::Action;
use keyboard_hook::types::ActionType::*;
use keyboard_hook::types::Key::*;
use keyboard_hook::types::Mapping;
use keyboard_hook::types::Modifier::*;
use keyboard_hook::types::SystemActionType;
use keyboard_hook::ActionHandler;
use keyboard_hook::KeyboardHook;
use std::fmt::Debug;
use std::sync::mpsc;

#[derive(PartialEq, Eq, Clone, Debug)]
#[allow(dead_code)]
enum MyActions {
    ToggleChannels,
    Volume,
}

use MyActions::*;

impl fmt::Display for MyActions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyActions::ToggleChannels => write!(f, "ToggleChannels"),
            MyActions::Volume => write!(f, "Volume"),
        }
    }
}

struct Handler;

impl ActionHandler<MyActions> for Handler {
    fn handle(&self, receiver: mpsc::Receiver<Action<MyActions>>) {
        for action in receiver {
            match action {
                Action::System(action) => match action {
                    SystemActionType::Hello => {
                        println!("Keyboard hooked. Press Alt+A -> E -> X -> I -> T to exit.")
                    }
                    SystemActionType::Bye => println!("Exiting..."),
                },
                Action::User(action, keys) => match action {
                    ToggleChannels => println!("ToggleChannels: {:?}", keys),
                    Volume => println!("Volume: {:?}", keys),
                },
            }
        }
    }
}

fn define_mappings() -> Vec<Vec<Mapping<MyActions>>> {
    vec![
        vec![
            t!(KeyA, ModAlt),
            t!(KeyE),
            t!(KeyX),
            t!(KeyI),
            a!(KeyT, System(SystemActionType::Bye)),
        ],
        vec![
            t!(KeyA, ModAlt),
            aat!(
                [key!(Key1), key!(Key2), key!(Key3), key!(Key4), key!(Key5)],
                User(ToggleChannels)
            ),
        ],
        vec![
            t!(KeyA, ModAlt),
            abt!([key!(KeyJ), key!(KeyK)], User(Volume)),
        ],
    ]
}

fn main() {
    let handler = Handler;
    let app = KeyboardHook::new(define_mappings(), Box::new(handler));

    if let Err(e) = app.hook() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
