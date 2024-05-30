use crate::mapping_trie::MappingTrie;
use crate::types::Mapping::*;
use crate::types::Modifier::*;
use crate::types::{Action, Modifier};
use crate::types::{Key, KeyPress};
use crate::windows::HookAction::{PassOn, Suppress};
use crate::windows::{HookAction, KeyboardHookManager, KeypressCallback};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const TIMEOUT_MS: u64 = 650;

struct SharedState {
    sender: mpsc::Sender<Action>,
    timeout_cancelled: bool,
    quitting: bool,
    timeout_retrigger: bool,
    timeout_running: bool,
    timeout_action: Option<Action>,
    buffer: Vec<KeyPress>,
}

/// KeypressHandler should determine if we can handle the key press by determining the action. If
/// the key press results in an action, we'll suppress propagating the key press event (Suppress),
/// otherwise we'll let other hooks handle it (PassOn).
pub struct KeypressHandler {
    state: Arc<(Mutex<SharedState>, Condvar)>,
    mapping_trie: MappingTrie,
}

impl KeypressHandler {
    pub fn new(sender: mpsc::Sender<Action>, mapping_trie: MappingTrie) -> KeypressHandler {
        KeypressHandler {
            state: Arc::new((
                Mutex::new(SharedState {
                    sender,
                    timeout_cancelled: false,
                    quitting: false,
                    timeout_retrigger: false,
                    timeout_running: false,
                    timeout_action: None,
                    buffer: vec![],
                }),
                Condvar::new(),
            )),
            mapping_trie,
        }
    }

    fn start_timeout(state_arc: Arc<(Mutex<SharedState>, Condvar)>) {
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
                    |s: &mut SharedState| {
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
                let timeout_action = state.timeout_action.clone();

                if let Some(action) = timeout_action {
                    println!("Sending action: {}", action);
                    state.timeout_action = None;
                    state.sender.send(action.clone()).unwrap();
                }

                state.buffer.clear();
                state.timeout_running = false;
                break;
            } else if state.timeout_cancelled {
                state.buffer.clear();
                state.timeout_running = false;
                break;
            }
        });
    }
}

impl KeypressCallback for KeypressHandler {
    fn handle(&mut self, key: u32, modifiers: &[Modifier]) -> HookAction {
        let modifier = if modifiers.contains(&ModAlt) {
            ModAlt
        } else {
            NoMod
        };

        // We don't care about Alt, Ctrl, Shift, Win alone. We only use these as modifiers.
        if [91, 160, 162, 164].contains(&key) {
            return PassOn;
        }

        let key_press = KeyPress::new(Key::from_u8(key as u8), modifier);
        let (mutex, condvar) = &*self.state;
        let mapping = {
            let state = mutex.lock().unwrap();
            let keypresses = &state.buffer;
            self.mapping_trie.find_mapping(keypresses, &key_press)
        };

        if let Some(mapping) = mapping {
            match mapping {
                Timeout(_) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.buffer.push(key_press);
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
                Action(_, action) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.buffer.push(key_press);
                    state.timeout_cancelled = true;
                    state.timeout_action = None;
                    let _ = state.sender.send(action.clone());

                    if Action::Bye == *action {
                        state.quitting = true;
                        KeyboardHookManager::stop_windows_loop();
                    }

                    condvar.notify_one();
                    state.buffer.clear();
                    drop(state);

                    return Suppress;
                }
                ActionBeforeTimeout(_, _) => return Suppress,
                ActionAfterTimeout(_, action) => {
                    println!("{}", mapping);
                    let mut state = mutex.lock().unwrap();
                    state.buffer.push(key_press);
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
        state.buffer.clear();
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
