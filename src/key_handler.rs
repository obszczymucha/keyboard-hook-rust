use std::sync::{mpsc, Arc, Mutex};
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
}

/// KeypressHandler should determine if we can handle the key press by determining the action. If
/// the key press results in an action, we'll suppress propagating the key press event (Suppress),
/// otherwise we'll let other hooks handle it (PassOn).
pub struct KeypressHandler {
    state: Arc<Mutex<SharedState>>,
}

impl KeypressHandler {
    pub fn new(sender: mpsc::Sender<Action>) -> KeypressHandler {
        KeypressHandler {
            state: Arc::new(Mutex::new(SharedState {
                sender,
                key_buffer: Vec::new(),
                waiting_for_next_key: false,
                quitting: false,
            })),
        }
    }

    fn send_toggles(state: &mut SharedState) {
        println!("Buffer: {:?}", state.key_buffer);

        state
            .sender
            .send(Action::ChannelToggles(state.key_buffer.clone()))
            .unwrap();

        state.key_buffer.clear();
    }

    fn handle_key(key: u32, state: &mut SharedState) -> HookAction {
        match key {
            KEY_X => {
                println!("Captured sequence: Alt+A -> X. Exiting...");
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
                Self::send_toggles(state);

                Suppress
            }
        }
    }
}

impl KeypressCallback for KeypressHandler {
    fn handle(&self, key: u32, modifiers: &Modifiers) -> HookAction {
        let mut state = self.state.lock().unwrap();
        let leader_pressed = key == LEADER_KEY && modifiers.left_alt;

        match (state.quitting, state.waiting_for_next_key, leader_pressed) {
            (true, _, _) | (false, false, false) => PassOn,
            (false, true, _) => Self::handle_key(key, &mut state),
            (false, _, true) => {
                state.waiting_for_next_key = true;
                println!("Leader key pressed.");

                let state_clone = Arc::clone(&self.state);

                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(TIMEOUT_MS));
                    let mut state = state_clone.lock().unwrap();

                    if state.waiting_for_next_key && !state.quitting {
                        println!("Timeout. Resetting...");
                        state.waiting_for_next_key = false;

                        Self::send_toggles(&mut state);
                    }
                });

                Suppress
            }
        }
    }
}
