use crate::keypress_buffer::KeyPressBuffer;
use crate::mapping_manager::find_mapping;
use crate::mapping_manager::ActionsOnTimeout;
use crate::mapping_trie::MappingTrie;
use crate::types::Key;
use crate::types::Modifier;
use crate::types::{Event, Modifier::*};
use crate::windows::HookAction::{PassOn, Suppress};
use crate::windows::{HookAction, KeypressCallback};
use crate::KeyPress;
use core::hash::Hash;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const TIMEOUT_MS: u64 = 650;

pub struct Buffers<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    pub key_buffer: KeyPressBuffer,
    pub actions_on_timeout: ActionsOnTimeout<A, T>,
}

impl<A, T> Buffers<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    pub fn new() -> Self {
        Self {
            key_buffer: KeyPressBuffer::new(),
            actions_on_timeout: ActionsOnTimeout::new(),
        }
    }
}

struct SharedState<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    sender: mpsc::Sender<Event<A, T>>,
    timeout_cancelled: bool,
    quitting: bool,
    timeout_retrigger: bool,
    timeout_running: bool,
    timeout_action: Option<A>,
    buffers: Buffers<A, T>,
}

/// KeypressHandler should determine if we can handle the key press by determining the action. If
/// the key press results in an action, we'll suppress propagating the key press event (Suppress),
/// otherwise we'll let other hooks handle it (PassOn).
pub(crate) struct KeypressHandler<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    state: Arc<(Mutex<SharedState<A, T>>, Condvar)>,
    mapping_trie: MappingTrie<A, T>,
}

impl<A, T> KeypressHandler<A, T>
where
    A: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    pub fn new(
        sender: mpsc::Sender<crate::types::Event<A, T>>,
        mapping_trie: MappingTrie<A, T>,
    ) -> KeypressHandler<A, T> {
        KeypressHandler {
            state: Arc::new((
                Mutex::new(SharedState {
                    sender,
                    timeout_cancelled: false,
                    quitting: false,
                    timeout_retrigger: false,
                    timeout_running: false,
                    timeout_action: None,
                    buffers: Buffers::new(),
                }),
                Condvar::new(),
            )),
            mapping_trie,
        }
    }

    fn start_timeout(state_arc: Arc<(Mutex<SharedState<A, T>>, Condvar)>) {
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
                    |s: &mut SharedState<A, T>| {
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

                if let Some(timeout_action) = &state.timeout_action {
                    state
                        .sender
                        .send(Event::Single(timeout_action.clone()))
                        .unwrap();
                }

                if let Some(tag) = state.buffers.actions_on_timeout.get_tag() {
                    state
                        .sender
                        .send(Event::Multi(
                            tag.clone(),
                            state
                                .buffers
                                .actions_on_timeout
                                .get_actions_on_timeout()
                                .clone(),
                        ))
                        .unwrap();
                }

                state.buffers.key_buffer.clear();
                state.buffers.actions_on_timeout.clear();
                state.timeout_action = None;
                state.timeout_running = false;
                break;
            } else if state.timeout_cancelled {
                state.buffers.key_buffer.clear();
                state.buffers.actions_on_timeout.clear();
                state.timeout_action = None;
                state.timeout_running = false;
                break;
            }
        });
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum KeyHandlerAction<A, T>
where
    A: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    Nothing, // TODO: Add reason for clarity.
    Timeout,
    SendAction(A),
    SendActionBeforeTimeout(A),
    SendActionOnTimeout(A),
    SendActionsOnTimeout(ActionsOnTimeout<A, T>),
    SendActionBeforeTimeoutAndOnTimeout {
        before: A,
        on: ActionsOnTimeout<A, T>,
    },
}

use KeyHandlerAction::*;

impl<A, T> Display for KeyHandlerAction<A, T>
where
    A: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nothing => write!(f, "Nothing"),
            Timeout => write!(f, "Timeout"),
            SendAction(action) => write!(f, "SendAction({})", action),
            SendActionBeforeTimeout(action) => {
                write!(f, "SendActionBeforeTimeout({})", action)
            }
            SendActionOnTimeout(action) => {
                write!(f, "SendActionOnTimeout({})", action)
            }
            SendActionBeforeTimeoutAndOnTimeout { before, on } => {
                write!(
                    f,
                    "SendActionBeforeTimeoutAndOnTimeout(before: {}, on: {})",
                    before, on
                )
            }
            SendActionsOnTimeout(actions) => write!(f, "SendActionsOnTimeout({})", actions),
        }
    }
}

impl<A, T> KeypressCallback for KeypressHandler<A, T>
where
    A: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
    T: 'static + PartialEq + Eq + Clone + Debug + Send + Sync + Display + Hash,
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

        let key_press = KeyPress::Mod(Key::from_u8(key as u8), modifier);
        let (mutex, condvar) = &*self.state;

        let handler_action = {
            let mut state = mutex.lock().unwrap();
            find_mapping(&key_press, &self.mapping_trie, &mut state.buffers)
        };

        match handler_action {
            Nothing => {
                println!("Nothing");
                let mut state = mutex.lock().unwrap();
                state.timeout_action = None;
                state.buffers.key_buffer.clear();
                state.buffers.actions_on_timeout.clear();
            }
            Timeout => {
                let mut state = mutex.lock().unwrap();
                state.timeout_action = None;
                state.buffers.key_buffer.clear();
                state.buffers.actions_on_timeout.clear();

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
            SendAction(ref action) => {
                let mut state = mutex.lock().unwrap();
                state.timeout_cancelled = true;
                state.timeout_action = None;
                state.buffers.key_buffer.clear();
                state.buffers.actions_on_timeout.clear();
                state.sender.send(Event::Single(action.clone())).unwrap();

                condvar.notify_one();
                drop(state);

                return Suppress;
            }
            SendActionBeforeTimeout(ref action) => {
                let mut state = mutex.lock().unwrap();
                state.sender.send(Event::Single(action.clone())).unwrap();

                state.timeout_action = None;
                state.buffers.actions_on_timeout.clear();

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
            SendActionOnTimeout(ref action) => {
                let mut state = mutex.lock().unwrap();
                state.timeout_action = Some(action.clone());
                state.buffers.actions_on_timeout.clear();

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
            SendActionsOnTimeout(_) => {
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
            SendActionBeforeTimeoutAndOnTimeout { ref before, .. } => {
                let mut state = mutex.lock().unwrap();
                state.sender.send(Event::Single(before.clone())).unwrap();
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
        }

        let mut state = mutex.lock().unwrap();
        state.buffers.actions_on_timeout.clear();

        if state.timeout_running {
            println!("Resetting.");
            state.timeout_cancelled = true;
            drop(state);
            condvar.notify_one();
        }

        PassOn
    }
}
