use crate::mapping_trie::MappingTrie;
use crate::types::Mapping::*;
use crate::types::Modifier::*;
use crate::types::{ActionType, Modifier};
use crate::types::{Key, KeyPress};
use crate::windows::HookAction::{PassOn, Suppress};
use crate::windows::{HookAction, KeyboardHookManager, KeypressCallback};
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const TIMEOUT_MS: u64 = 650;

struct SharedState<T>
where
    T: PartialEq + Eq + Clone + Debug + Send + Sync + Display,
{
    sender: mpsc::Sender<ActionType<T>>,
    timeout_cancelled: bool,
    quitting: bool,
    timeout_retrigger: bool,
    timeout_running: bool,
    timeout_action: Option<ActionType<T>>,
    mapping_trie: MappingTrie<T>,
}

/// KeypressHandler should determine if we can handle the key press by determining the action. If
/// the key press results in an action, we'll suppress propagating the key press event (Suppress),
/// otherwise we'll let other hooks handle it (PassOn).
pub struct KeypressHandler<T>
where
    T: PartialEq + Eq + Clone + Debug + Send + Sync + Display,
{
    state: Arc<(Mutex<SharedState<T>>, Condvar)>,
}

impl<T> KeypressHandler<T>
where
    T: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display,
{
    pub fn new(
        sender: mpsc::Sender<ActionType<T>>,
        mapping_trie: MappingTrie<T>,
    ) -> KeypressHandler<T> {
        KeypressHandler {
            state: Arc::new((
                Mutex::new(SharedState {
                    sender,
                    timeout_cancelled: false,
                    quitting: false,
                    timeout_retrigger: false,
                    timeout_running: false,
                    timeout_action: None,
                    mapping_trie,
                }),
                Condvar::new(),
            )),
        }
    }

    fn start_timeout(state_arc: Arc<(Mutex<SharedState<T>>, Condvar)>) {
        let (mutex, _) = &*state_arc;
        let state = mutex.lock().unwrap();

        if state.timeout_running {
            return;
        }

        drop(state);

        let cloned_state = Arc::clone(&state_arc);
        thread::spawn(move || loop {
            let (mutex, condvar) = &*cloned_state;
            let mut state = mutex.lock().unwrap();
            state.timeout_running = true;
            state.timeout_retrigger = false;
            state.timeout_cancelled = false;

            let (lock_result, timeout_result) = condvar
                .wait_timeout_while(
                    state,
                    Duration::from_millis(TIMEOUT_MS),
                    |s: &mut SharedState<T>| {
                        !s.timeout_retrigger && !s.quitting && !s.timeout_cancelled
                    },
                )
                .unwrap();

            let mut state = lock_result;

            if state.quitting {
                state.timeout_running = false;
                return;
            }

            if timeout_result.timed_out() && !state.timeout_retrigger && !state.timeout_cancelled {
                println!("Timeout!");
                state.mapping_trie.reset();
                let timeout_action = state.timeout_action.clone();

                if let Some(action) = timeout_action {
                    state.timeout_action = None;
                    state.sender.send(action.clone()).unwrap();
                }

                state.timeout_running = false;
                break;
            } else if state.timeout_cancelled {
                state.timeout_running = false;
                break;
            }
        });
    }
}

impl<T> KeypressCallback for KeypressHandler<T>
where
    T: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display,
{
    fn handle(&mut self, key: u32, modifiers: &[Modifier]) -> HookAction {
        let modifier = if modifiers.contains(&ModAlt) {
            ModAlt
        } else {
            NoMod
        };

        // We don't care about Alt, Ctrl, Shift, Win alone. We only use these as modifiers.
        if [91, 92, 93, 160, 161, 162, 163, 164, 165].contains(&key) {
            return PassOn;
        }

        let key_press = KeyPress::new(Key::from_u8(key as u8), modifier);
        let (mutex, condvar) = &*self.state;
        let mapping = {
            let mut state = mutex.lock().unwrap();
            // let keypresses = &state.buffer;
            state.mapping_trie.find_mapping(&key_press)
        };

        if let Some(mapping) = mapping {
            match mapping {
                Timeout(_) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.timeout_action = None;

                    if state.timeout_running {
                        state.timeout_retrigger = true;
                        drop(state);
                        condvar.notify_one();
                    } else {
                        drop(state);
                        Self::start_timeout(Arc::clone(&self.state));
                    }

                    return Suppress;
                }
                Action(_, ref action) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.timeout_cancelled = true;
                    state.timeout_action = None;
                    state.mapping_trie.reset();
                    let _ = state.sender.send(action.clone());

                    if ActionType::Bye == *action {
                        state.quitting = true;
                        KeyboardHookManager::stop_windows_loop();
                    }

                    condvar.notify_one();
                    drop(state);

                    return Suppress;
                }
                ActionBeforeTimeout(_, ref action) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.sender.send(action.clone()).unwrap();
                    state.timeout_action = None;

                    if state.timeout_running {
                        state.timeout_retrigger = true;
                        drop(state);
                        condvar.notify_one();
                    } else {
                        drop(state);
                        Self::start_timeout(Arc::clone(&self.state));
                    }

                    return Suppress;
                }
                ActionAfterTimeout(_, ref action) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.timeout_action = Some(action.clone());

                    if state.timeout_running {
                        state.timeout_retrigger = true;
                        drop(state);
                        condvar.notify_one();
                    } else {
                        drop(state);
                        Self::start_timeout(Arc::clone(&self.state));
                    }

                    return Suppress;
                }
            }
        }

        let mut state = mutex.lock().unwrap();
        state.timeout_action = None;

        if state.timeout_running {
            println!("Resetting.");
            state.timeout_cancelled = true;
            drop(state);
            condvar.notify_one();
        }

        PassOn
    }
}
