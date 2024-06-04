pub mod action_handler;
mod key_handler;
pub mod macros;
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
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<ShutdownAction>();

        let handler = self.handler.clone();
        let consumer_handle = thread::spawn(move || {
            handler.handle(rx, shutdown_tx);
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

        let shutdown_handle = thread::spawn(move || {
            for action in shutdown_rx {
                match action {
                    ShutdownAction => {
                        KeyboardHookManager::stop_windows_loop();
                        break;
                    }
                }
            }
        });

        producer_handle.join().unwrap();
        shutdown_handle.join().unwrap();
        consumer_handle.join().unwrap();

        Ok(())
    }
}
