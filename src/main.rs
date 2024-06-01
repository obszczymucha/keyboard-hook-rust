mod action_handler;
mod key_handler;
mod mapping_trie;
mod types;
mod windows;

use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{mpsc, Arc};
use std::thread;

use crate::types::ActionType::*;
use crate::types::Key::*;
use crate::types::Modifier::*;
use crate::types::{ActionType, Mapping};
use action_handler::ActionHandler;
use key_handler::KeypressHandler;
use mapping_trie::MappingTrie;

use crate::windows::KeyboardHookManager;

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
    fn consume(&self, receiver: mpsc::Receiver<ActionType<MyActions>>) {
        for action in receiver {
            match action {
                Hello => {
                    println!("Keyboard hooked. Press Alt+A -> E -> X -> I -> T to exit.")
                }
                User(my_action) => match my_action {
                    ToggleChannels => println!("ToggleChannels"),
                    Volume => {
                        self.api.do_something();
                        println!("Volume");
                    }
                },
                Bye => println!("Exiting..."),
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
            a!(KeyT, ActionType::Bye),
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
    let app = App::new(define_mappings(), Box::new(handler));

    if let Err(e) = app.hook() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

struct App<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    handler: Arc<Box<dyn ActionHandler<T> + Send + Sync>>,
    mappings: Arc<Vec<Vec<Mapping<T>>>>,
}

impl<T> App<T>
where
    T: 'static + PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn new(
        mappings: Vec<Vec<Mapping<T>>>,
        handler: Box<dyn ActionHandler<T> + Send + Sync>,
    ) -> Self {
        Self {
            handler: Arc::new(handler),
            mappings: Arc::new(mappings),
        }
    }

    fn hook(&self) -> Result<(), &str> {
        let (tx, rx) = mpsc::channel::<ActionType<T>>();

        let handler = self.handler.clone();
        let consumer_handle = thread::spawn(move || {
            handler.consume(rx);
        });

        let mappings = self.mappings.clone();
        let producer_handle = thread::spawn(move || {
            let mut manager = KeyboardHookManager::new()?;
            let handler = Box::new(KeypressHandler::new(
                tx.clone(),
                MappingTrie::from_mappings(&mappings),
            ));
            manager.hook(tx.clone(), handler)
        });

        let _ = producer_handle.join().unwrap();
        consumer_handle.join().unwrap();
        Ok(())
    }
}
