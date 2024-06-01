mod action_handler;
mod key_handler;
mod mapping_trie;
mod types;
mod windows;

use std::sync::{mpsc, Arc};
use std::thread;

use action_handler::ActionHandler;
use key_handler::KeypressHandler;
use mapping_trie::{define_mappings, MappingTrie};
use types::{ActionType, Mapping};

use crate::windows::KeyboardHookManager;

fn main() {
    let handler = ActionHandler;
    let app = App::new(define_mappings(), handler);

    if let Err(e) = app.hook() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

struct App {
    handler: Arc<ActionHandler>,
    mappings: Arc<Vec<Vec<Mapping>>>,
}

impl App {
    fn new(mappings: Vec<Vec<Mapping>>, handler: ActionHandler) -> Self {
        Self {
            handler: Arc::new(handler),
            mappings: Arc::new(mappings),
        }
    }

    fn hook(&self) -> Result<(), &str> {
        let (tx, rx) = mpsc::channel::<ActionType>();

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
