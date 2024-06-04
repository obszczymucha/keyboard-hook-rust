use crate::types::Event;
use crate::ShutdownAction;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::mpsc;

// #[allow(dead_code)]
// fn deduplicate(key_presses: &KeyPressMappings) -> KeyPressMappings {
//     let mut result = HashSet::new();
//
//     for key_press in key_presses.field1.iter() {
//         if result.contains(key_press) {
//             result.remove(key_press);
//         } else {
//             result.insert(key_press.clone());
//         }
//     }
//
//     KeyPressMappings { field1: result.into_iter().collect() }
// }
//
pub trait ActionHandler<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
{
    fn handle(&self, receiver: mpsc::Receiver<Event<A, T>>, sender: mpsc::Sender<ShutdownAction>);
}
