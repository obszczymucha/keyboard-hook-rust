mod action_handler;
mod key_handler;
mod types;
mod windows;

use std::sync::mpsc;
use std::thread;

use action_handler::ActionHandler;
use key_handler::KeypressHandler;
use types::Action;

use crate::windows::KeyboardHookManager;

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

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let (tx, rx) = mpsc::channel::<Action>();

    let consumer_handle = thread::spawn(|| {
        let consumer = ActionHandler::new(rx);
        consumer.start();
    });

    let producer_handle = thread::spawn(move || {
        let mut manager = KeyboardHookManager::new()?;
        let handler = Box::new(KeypressHandler::new(tx.clone()));
        manager.hook(tx.clone(), handler)
    });

    let _ = producer_handle.join().unwrap();
    consumer_handle.join().unwrap();
    Ok(())
}
