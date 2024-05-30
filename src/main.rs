mod action_handler;
mod key_handler;
mod mapping_trie;
mod types;
mod windows;

use std::sync::mpsc;
use std::thread;

use action_handler::ActionHandler;
use key_handler::KeypressHandler;
use mapping_trie::{define_mappings, MappingTrie};
use types::Action;

use crate::windows::KeyboardHookManager;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let (tx, rx) = mpsc::channel::<Action>();

    let consumer_handle = thread::spawn(|| {
        ActionHandler::consume(rx);
    });

    let producer_handle = thread::spawn(move || {
        let mut manager = KeyboardHookManager::new()?;
        let handler = Box::new(KeypressHandler::new(
            tx.clone(),
            MappingTrie::from_mappings(define_mappings()),
        ));
        manager.hook(tx.clone(), handler)
    });

    let _ = producer_handle.join().unwrap();
    consumer_handle.join().unwrap();
    Ok(())
}
