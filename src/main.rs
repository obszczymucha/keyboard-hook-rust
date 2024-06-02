mod action_handler;
mod key_handler;
mod keyboard_hook;
mod mapping_trie;
mod types;
mod windows;

use crate::types::Action;
use crate::types::ActionType::*;
use crate::types::Key::*;
use crate::types::Mapping;
use crate::types::Modifier::*;
use action_handler::ActionHandler;
use core::fmt;
use keyboard_hook::KeyboardHook;
use std::fmt::Debug;
use std::sync::mpsc;
use types::SystemActionType;

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

#[allow(dead_code)]
struct MyApi {
    dupa: bool,
}

impl MyApi {
    fn new() -> Self {
        Self { dupa: true }
    }

    fn do_something(&self) {}
}

struct Handler {
    api: MyApi,
}

impl Handler {
    fn new(api: MyApi) -> Self {
        Self { api }
    }
}

impl ActionHandler<MyActions> for Handler {
    fn consume(&self, receiver: mpsc::Receiver<Action<MyActions>>) {
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
                    Volume => {
                        self.api.do_something();
                        println!("Volume: {:?}", keys);
                    }
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
    let api = MyApi::new();
    let handler = Handler::new(api);
    let app = KeyboardHook::new(define_mappings(), Box::new(handler));

    if let Err(e) = app.hook() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
