use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetKeyState, GetMessageW, PostQuitMessage, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_SYSKEYDOWN,
};
// use winapi::um::winuser::{
//     VK_LCONTROL, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN,
// };
use winapi::um::winuser::VK_LMENU;

const LEADER_KEY: u32 = b'A' as u32;
const FOLLOWUP_KEY: u32 = b'X' as u32;
const TIMEOUT_MS: u64 = 650;
const KEY_PRESSED_MASK: u16 = 0x8000;

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

struct Modifiers {
    // left_control: bool,
    // right_control: bool,
    // left_shift: bool,
    // right_shift: bool,
    // left_win: bool,
    // right_win: bool,
    left_alt: bool,
    // right_alt: bool,
}

fn handle_key_press(vk_code: u32, modifiers: &Modifiers) -> bool {
    if WAITING_FOR_NEXT_KEY.load(Ordering::SeqCst) {
        if vk_code == FOLLOWUP_KEY {
            println!("Captured sequence: Alt+A -> X. Exiting...");
            unsafe {
                PostQuitMessage(0);
            }
        } else {
            println!("No mapping for {}. Resetting...", vk_code as u8 as char);
            WAITING_FOR_NEXT_KEY.store(false, Ordering::SeqCst);
        }
        return true;
    }

    if vk_code == LEADER_KEY && modifiers.left_alt == true {
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

static mut HOOK: HHOOK = std::ptr::null_mut();
struct KeyboardHookManager;

impl KeyboardHookManager {
    fn new() -> Result<Self, &'static str> {
        unsafe {
            HOOK = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                GetModuleHandleW(std::ptr::null()),
                0,
            );
            if HOOK.is_null() {
                return Err("Failed to install keyboard hook.");
            }
        }
        Ok(Self)
    }
}

impl Drop for KeyboardHookManager {
    fn drop(&mut self) {
        unsafe {
            UnhookWindowsHookEx(HOOK);
        }
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code != 0 || (w_param != WM_KEYDOWN as WPARAM && w_param != WM_SYSKEYDOWN as WPARAM) {
        return CallNextHookEx(HOOK, n_code, w_param, l_param);
    }

    let p_keyboard: &KBDLLHOOKSTRUCT = &*(l_param as *const KBDLLHOOKSTRUCT);
    let modifiers = Modifiers {
        // left_control: (GetKeyState(VK_LCONTROL) as u16 & KEY_PRESSED_MASK) != 0,
        // right_control: (GetKeyState(VK_RCONTROL) as u16 & KEY_PRESSED_MASK) != 0,
        // left_shift: (GetKeyState(VK_LSHIFT) as u16 & KEY_PRESSED_MASK) != 0,
        // right_shift: (GetKeyState(VK_RSHIFT) as u16 & KEY_PRESSED_MASK) != 0,
        // left_win: (GetKeyState(VK_LWIN) as u16 & KEY_PRESSED_MASK) != 0,
        // right_win: (GetKeyState(VK_RWIN) as u16 & KEY_PRESSED_MASK) != 0,
        left_alt: (GetKeyState(VK_LMENU) as u16 & KEY_PRESSED_MASK) != 0,
        // right_alt: (GetKeyState(VK_RMENU) as u16 & KEY_PRESSED_MASK) != 0,
    };

    if handle_key_press(p_keyboard.vkCode, &modifiers) == true {
        return 1;
    } else {
        return CallNextHookEx(HOOK, n_code, w_param, l_param);
    }
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
    let _keyboard_hook_manager = KeyboardHookManager::new()?;

    println!("Keyboard hooked. Press Alt+A and then X to exit.");

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}
