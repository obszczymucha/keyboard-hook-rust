use core::fmt;
use keyboard_hook::a;
use keyboard_hook::aot;
use keyboard_hook::t;
use keyboard_hook::types::Event;
use keyboard_hook::types::Key::*;
use keyboard_hook::types::Mapping;
use keyboard_hook::types::Modifier::*;
use keyboard_hook::types::SystemAction;
use keyboard_hook::types::TerminateHook;
use keyboard_hook::ActionHandler;
use keyboard_hook::KeyboardHook;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::mpsc;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
#[allow(dead_code)]
enum MyActions {
    ToggleChannels,
    Volume,
    Terminate,
}

use MyActions::*;

impl Display for MyActions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyActions::ToggleChannels => write!(f, "ToggleChannels"),
            MyActions::Volume => write!(f, "Volume"),
            MyActions::Terminate => write!(f, "OnShutdown"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
#[allow(dead_code)]
enum MyTags {}

impl Display for MyTags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

struct Handler;

impl ActionHandler<MyActions, MyTags> for Handler {
    fn handle(
        &self,
        receiver: mpsc::Receiver<Event<MyActions, MyTags>>,
        sender: mpsc::Sender<TerminateHook>,
    ) {
        for action in receiver {
            match action {
                Event::System(SystemAction::KeyboardHooked) => {
                    println!("Keyboard hooked. Press Alt+A -> E -> X -> I -> T to exit.")
                }
                Event::System(SystemAction::KeyboardUnhooked) => println!("Exiting..."),
                Event::Single(Terminate) => sender.send(TerminateHook).unwrap(),
                Event::Single(action) => println!("Received action: {}", action),
                Event::Multi(tag, actions) => {
                    println!("Received actions ({:?}): {:?}", tag, actions)
                }
            }
        }
    }
}

fn define_mappings() -> Vec<Vec<Mapping<MyActions, MyTags>>> {
    vec![
        // Alt+A -> E -> X -> I -> T
        vec![
            t!(KeyA, ModAlt),
            t!(KeyE),
            t!(KeyX),
            t!(KeyI),
            a!(KeyT, Terminate), // Immediate action.
        ],
        // Alt+A -> Q
        vec![
            t!(KeyA, ModAlt),
            aot!(KeyQ, Terminate), // Action on timeout.
        ],
        // vec![
        //     t!(KeyA, ModAlt),
        //     tm!(
        //         [
        //             key_a!(Key1, ToggleChannel1),
        //             key_a!(Key2, ToggleChannel2),
        //             key_a!(Key3, ToggleChannel3),
        //             key_a!(Key4, ToggleChannel4),
        //             key_a!(Key5, ToggleChannel5)
        //         ],
        //         ToggleChannels
        //     ),
        // ],
        // vec![
        //     t!(KeyA, ModAlt),
        //     abt!([key!(KeyJ), key!(KeyK)], User(Volume)),
        // ],
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
