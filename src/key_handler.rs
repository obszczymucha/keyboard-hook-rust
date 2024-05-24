use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use crate::types::{Action, Modifiers};
use crate::windows::HookAction::{PassOn, Suppress};
use crate::windows::{HookAction, KeyboardHookManager, KeypressCallback};

use std::sync::atomic::{AtomicBool, Ordering};

static WAITING_FOR_NEXT_KEY: AtomicBool = AtomicBool::new(false);

const LEADER_KEY: u32 = b'A' as u32;
const FOLLOWUP_KEY: u32 = b'X' as u32;
const TIMEOUT_MS: u64 = 650;

/// KeypressHandler should determine if we can handle the key press by determining the action. If
/// the key press results in an action, we'll suppress propagating the key press event (Suppress),
/// otherwise we'll let other hooks handle it (PassOn).
pub struct KeypressHandler {
    sender: mpsc::Sender<Action>,
}

impl KeypressHandler {
    pub fn new(sender: mpsc::Sender<Action>) -> KeypressHandler {
        KeypressHandler { sender }
    }
}

impl KeypressCallback for KeypressHandler {
    fn handle(&self, key: u32, modifiers: &Modifiers) -> HookAction {
        if WAITING_FOR_NEXT_KEY.load(Ordering::SeqCst) {
            if key == FOLLOWUP_KEY {
                println!("Captured sequence: Alt+A -> X. Exiting...");
                self.sender.send(Action::Bye).unwrap();
                KeyboardHookManager::stop_windows_loop();
                return Suppress;
            } else if key == b'Q' as u32 {
                self.sender.send(Action::Hello).unwrap();
                return Suppress;
            } else {
                println!("No mapping for {}. Resetting...", key as u8 as char);
                WAITING_FOR_NEXT_KEY.store(false, Ordering::SeqCst);
                return Suppress;
            }
        }

        if key == LEADER_KEY && modifiers.left_alt {
            WAITING_FOR_NEXT_KEY.store(true, Ordering::SeqCst);
            println!("Leader key pressed.");

            let waiting_for_next_key = Arc::new(AtomicBool::new(true));
            let waiting_for_next_key_clone = Arc::clone(&waiting_for_next_key);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(TIMEOUT_MS));
                if waiting_for_next_key_clone.load(Ordering::SeqCst) {
                    println!("Timeout. Resetting...");
                    waiting_for_next_key_clone.store(false, Ordering::SeqCst);
                }
            });

            return Suppress;
        }

        PassOn
    }
}
