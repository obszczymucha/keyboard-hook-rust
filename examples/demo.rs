use core::fmt;
use keyboard_hook::a;
use keyboard_hook::aot;
use keyboard_hook::c;
use keyboard_hook::key_a;
use keyboard_hook::key_aot;
use keyboard_hook::shutdown;
use keyboard_hook::t;
use keyboard_hook::types::Event;
use keyboard_hook::types::Key::*;
use keyboard_hook::types::Mapping;
use keyboard_hook::types::Modifier::*;
use keyboard_hook::types::SystemAction;
use keyboard_hook::ActionHandler;
use keyboard_hook::KeyboardHook;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::mpsc;
use SystemAction::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
#[allow(dead_code)]
enum MyActions {
    Princess,
    Kenny,
    VolumeUp,
    VolumeDown,
    ToggleChannel1,
    ToggleChannel2,
    ToggleChannel3,
    ToggleChannel4,
    ToggleChannel5,
    UseStrip2,
}

use MyActions::*;

impl Display for MyActions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyActions::Princess => write!(f, "Princess"),
            MyActions::Kenny => write!(f, "Kenny"),
            MyActions::VolumeUp => write!(f, "VolumeUp"),
            MyActions::VolumeDown => write!(f, "VolumeDown"),
            MyActions::ToggleChannel1 => write!(f, "ToggleChannel1"),
            MyActions::ToggleChannel2 => write!(f, "ToggleChannel2"),
            MyActions::ToggleChannel3 => write!(f, "ToggleChannel3"),
            MyActions::ToggleChannel4 => write!(f, "ToggleChannel4"),
            MyActions::ToggleChannel5 => write!(f, "ToggleChannel5"),
            MyActions::UseStrip2 => write!(f, "UseStrip2"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
#[allow(dead_code)]
enum MyTags {
    ToggleChannels,
    Volume,
}

use MyTags::*;

impl Display for MyTags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

struct Handler;

impl ActionHandler<MyActions, MyTags> for Handler {
    fn handle(&self, receiver: mpsc::Receiver<Event<MyActions, MyTags>>) {
        for action in receiver {
            match action {
                Event::System(KeyboardHooked) => {
                    println!("Hello. Press Alt+A -> E -> X -> I -> T to exit.")
                }
                Event::System(KeyboardUnhooked) => println!("Exiting..."),
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
            shutdown!(KeyT), // Unhook the keyboard and exit.
        ],
        // Alt+A -> Q
        vec![
            t!(KeyA, ModAlt),
            aot!(KeyQ, Princess), // Action on timeout.
        ],
        // Alt+A -> W
        vec![
            t!(KeyA, ModAlt),
            a!(KeyW, Kenny), // Immediate action.
        ],
        // Alt+A -> S -> 2 -> [12345]*
        vec![
            t!(KeyA, ModAlt),
            t!(KeyS),
            aot!(Key2, UseStrip2),
            c!(
                [
                    key_aot!(Key1, ToggleChannel1),
                    key_aot!(Key2, ToggleChannel2),
                    key_aot!(Key3, ToggleChannel3),
                    key_aot!(Key4, ToggleChannel4),
                    key_aot!(Key5, ToggleChannel5)
                ],
                ToggleChannels
            ),
        ],
        // Alt+A -> [12345]*
        vec![
            t!(KeyA, ModAlt),
            c!(
                [
                    key_aot!(Key1, ToggleChannel1),
                    key_aot!(Key2, ToggleChannel2),
                    key_aot!(Key3, ToggleChannel3),
                    key_aot!(Key4, ToggleChannel4),
                    key_aot!(Key5, ToggleChannel5)
                ],
                ToggleChannels
            ),
        ],
        // Alt+A -> [JK]*
        vec![
            t!(KeyA, ModAlt),
            c!([key_a!(KeyJ, VolumeDown), key_a!(KeyK, VolumeUp)], Volume),
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
