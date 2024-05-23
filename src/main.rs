mod types;
mod windows;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use types::Modifiers;

use crate::windows::{quit_windows_app, KeyboardHookManager};

const LEADER_KEY: u32 = b'A' as u32;
const FOLLOWUP_KEY: u32 = b'X' as u32;
const TIMEOUT_MS: u64 = 650;

// const MOD_CONTROL: u8 = 0x0002;
// const MOD_SHIFT: u8 = 0x0004;
// const MOD_WIN: u8 = 0x0008;
// const MOD_ALT: u8 = 0x0001;

// #[repr(u8)]
// enum Modifier {
// None = 0,
// Control = MOD_CONTROL,
// Shift = MOD_SHIFT,
// Win = MOD_WIN,
// Alt = MOD_ALT,
// ControlShift = MOD_CONTROL | MOD_SHIFT,
// ControlWin = MOD_CONTROL | MOD_WIN,
// ControlAlt = MOD_CONTROL | MOD_ALT,
// ShiftWin = MOD_SHIFT | MOD_WIN,
// ShiftAlt = MOD_SHIFT | MOD_ALT,
// WinAlt = MOD_WIN | MOD_ALT,
// ControlShiftWin = MOD_CONTROL | MOD_SHIFT | MOD_WIN,
// ControlShiftAlt = MOD_CONTROL | MOD_SHIFT | MOD_ALT,
// ControlWinAlt = MOD_CONTROL | MOD_WIN | MOD_ALT,
// ShiftWinAlt = MOD_SHIFT | MOD_WIN | MOD_ALT,
// All = MOD_CONTROL | MOD_SHIFT | MOD_WIN | MOD_ALT,
// }

// struct LeaderKey {
//     code: u32,
// }
//
// struct KeySequence<'a> {
//     leader: LeaderKey,
//     next: Key<'a>,
// }
//
// enum Action {
//     Exit,
// }
//
// enum Key<'a> {
//     FollowUp { code: u32, next: &'a FollowUp<'a> },
//     Final { code: u32, action: Action },
// }
//
// enum FollowUp<'a> {
//     Next { code: u32, next: &'a FollowUp<'a> },
//     Final { code: u32, action: Action },
// }

static WAITING_FOR_NEXT_KEY: AtomicBool = AtomicBool::new(false);

fn handle_key_press(vk_code: u32, modifiers: &Modifiers) -> bool {
    if WAITING_FOR_NEXT_KEY.load(Ordering::SeqCst) {
        if vk_code == FOLLOWUP_KEY {
            println!("Captured sequence: Alt+A -> X. Exiting...");
            quit_windows_app();
        } else {
            println!("No mapping for {}. Resetting...", vk_code as u8 as char);
            WAITING_FOR_NEXT_KEY.store(false, Ordering::SeqCst);
        }
        return true;
    }

    if vk_code == LEADER_KEY && modifiers.left_alt {
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

        return true;
    }

    false
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    // The variable has to be here to keep the scope.
    // If we remove it, the destructor is called immediately. Fun.
    let mut manager = KeyboardHookManager::new()?;
    manager.hook(handle_key_press)?;

    println!("Keyboard hooked. Press Alt+A and then X to exit.");
    Ok(())
}
