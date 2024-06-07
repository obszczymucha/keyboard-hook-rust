pub mod action_handler;
mod key_handler;
mod keypress_buffer;
pub mod macros;
mod mapping_manager;
mod mapping_trie;
pub mod types;
mod windows;

pub use crate::action_handler::ActionHandler;
use crate::key_handler::KeypressHandler;
use crate::mapping_trie::MappingTrie;
use crate::types::*;
use crate::windows::KeyboardHookManager;
use core::hash::Hash;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{mpsc, Arc};
use std::thread;

pub struct KeyboardHook<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    handler: Arc<Box<dyn ActionHandler<A, T> + Send + Sync>>,
    mappings: Arc<Vec<Vec<Mapping<A, T>>>>,
}

impl<A, T> KeyboardHook<A, T>
where
    A: 'static + PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
    T: 'static + PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
{
    pub fn new(
        mappings: Vec<Vec<Mapping<A, T>>>,
        handler: Box<dyn ActionHandler<A, T> + Send + Sync>,
    ) -> Self {
        Self {
            handler: Arc::new(handler),
            mappings: Arc::new(mappings),
        }
    }

    pub fn hook(&self) -> Result<(), &str> {
        let (tx, rx) = mpsc::channel::<Event<A, T>>();

        let handler = self.handler.clone();
        let consumer_handle = thread::spawn(move || {
            handler.handle(rx);
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
