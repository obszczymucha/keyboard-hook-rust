use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use crate::types::{Action, Modifiers};
use crate::windows::HookAction::{PassOn, Suppress};
use crate::windows::{HookAction, KeyboardHookManager, KeypressCallback};

const LEADER_KEY: u32 = b'A' as u32;
const KEY_X: u32 = b'X' as u32;
const TIMEOUT_MS: u64 = 650;

struct SharedState {
    sender: mpsc::Sender<Action>,
    key_buffer: Vec<u32>,
    waiting_for_next_key: bool,
    quitting: bool,
    timeout_triggered: bool,
}

/// KeypressHandler should determine if we can handle the key press by determining the action. If
/// the key press results in an action, we'll suppress propagating the key press event (Suppress),
/// otherwise we'll let other hooks handle it (PassOn).
pub struct KeypressHandler {
    state: Arc<(Mutex<SharedState>, Condvar)>,
}

impl KeypressHandler {
    pub fn new(sender: mpsc::Sender<Action>) -> KeypressHandler {
        KeypressHandler {
            state: Arc::new((
                Mutex::new(SharedState {
                    sender,
                    key_buffer: Vec::new(),
                    waiting_for_next_key: false,
                    quitting: false,
                    timeout_triggered: false,
                }),
                Condvar::new(),
            )),
        }
    }

    fn reset_timeout(state_arc: Arc<(Mutex<SharedState>, Condvar)>) {
        let cloned_state = Arc::clone(&state_arc);
        thread::spawn(move || loop {
            let (mutex, condvar) = &*cloned_state;
            let mut state = mutex.lock().unwrap();
            state.timeout_triggered = false;

            let (lock_result, timeout_result) = condvar
                .wait_timeout_while(
                    state,
                    Duration::from_millis(TIMEOUT_MS),
                    |s: &mut SharedState| !s.timeout_triggered && !s.quitting,
                )
                .unwrap();

            let mut state = lock_result;

            if state.quitting {
                break;
            }

            if timeout_result.timed_out() && !state.timeout_triggered {
                println!("Timeout. Resetting...");
                state.waiting_for_next_key = false;
                Self::send_toggles(&mut state);
                state.timeout_triggered = true;
                break;
            }
        });
    }

    fn send_toggles(state: &mut SharedState) {
        println!("Buffer: {:?}", state.key_buffer);

        state
            .sender
            .send(Action::ChannelToggles(state.key_buffer.clone()))
            .unwrap();

        state.key_buffer.clear();
    }

    fn handle_key(key: u32, state_arc: Arc<(Mutex<SharedState>, Condvar)>) -> HookAction {
        let (mutex, _) = &*state_arc;
        let mut state = mutex.lock().unwrap();

        match key {
            KEY_X => {
                state.sender.send(Action::Bye).unwrap();
                state.quitting = true;
                KeyboardHookManager::stop_windows_loop();

                Suppress
            }
            49..=53 => {
                state.key_buffer.push(key);

                Suppress
            }
            _ => {
                println!(
                    "No mapping for {} ({}). Resetting...",
                    key as u8 as char, key
                );

                state.waiting_for_next_key = false;
                Self::send_toggles(&mut state);

                Suppress
            }
        }
    }

    fn handle_leader_key(state_arc: Arc<(Mutex<SharedState>, Condvar)>) -> HookAction {
        let (mutex, _) = &*state_arc;
        let mut state = mutex.lock().unwrap();
        state.waiting_for_next_key = true;
        println!("Leader key pressed.");
        drop(state);

        Self::reset_timeout(state_arc);
        Suppress
    }
}

impl KeypressCallback for KeypressHandler {
    fn handle(&self, key: u32, modifiers: &Modifiers) -> HookAction {
        let (quitting, waiting, leader_pressed) = {
            let (mutex, _) = &*self.state;
            let state = mutex.lock().unwrap();
            let leader_pressed = key == LEADER_KEY && modifiers.left_alt;
            (state.quitting, state.waiting_for_next_key, leader_pressed)
        };

        match (quitting, waiting, leader_pressed) {
            (true, _, _) | (false, false, false) => PassOn,
            (_, true, _) => {
                let result = Self::handle_key(key, Arc::clone(&self.state));
                let (mutex, condvar) = &*self.state;
                let mut state = mutex.lock().unwrap();
                state.timeout_triggered = true;
                drop(state);
                condvar.notify_one();
                result
            }
            (_, _, true) => Self::handle_leader_key(Arc::clone(&self.state)),
        }
    }
}
