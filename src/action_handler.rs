use crate::types::{ActionType, KeyPresses};
use std::fmt::Debug;
use std::fmt::Display;
use std::{collections::HashSet, sync::mpsc};

pub struct MyActionHandler;

#[allow(dead_code)]
fn deduplicate(key_presses: &KeyPresses) -> KeyPresses {
    let mut result = HashSet::new();

    for key_press in key_presses.0.iter() {
        if result.contains(key_press) {
            result.remove(key_press);
        } else {
            result.insert(key_press.clone());
        }
    }

    KeyPresses(result.into_iter().collect())
}

pub trait ActionHandler<T>
where
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn consume(&self, receiver: mpsc::Receiver<ActionType<T>>);
}
