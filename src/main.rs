use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::UnregisterHotKey;
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetKeyState, GetMessageW, PostQuitMessage, RegisterHotKey,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MOD_ALT, MSG,
    VK_MENU, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

const LEADER_KEY: u32 = b'A' as u32;
const FOLLOWUP_KEY: u32 = b'X' as u32;
const TIMEOUT_MS: u64 = 500;
const KEY_PRESSED_MASK: u16 = 0x8000;

static mut HOOK: HHOOK = std::ptr::null_mut();
static WAITING_FOR_NEXT_KEY: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code != 0 || (w_param != WM_KEYDOWN as WPARAM && w_param != WM_SYSKEYDOWN as WPARAM) {
        return CallNextHookEx(HOOK, n_code, w_param, l_param);
    }

    let p_keyboard = &*(l_param as *const KBDLLHOOKSTRUCT);

    if WAITING_FOR_NEXT_KEY.load(Ordering::SeqCst) {
        if p_keyboard.vkCode == FOLLOWUP_KEY {
            println!("Captured sequence: Alt+A -> X. Exiting...");
            PostQuitMessage(0);
        } else {
            println!(
                "No mapping for {}. Resetting...",
                p_keyboard.vkCode as u8 as char
            );
            WAITING_FOR_NEXT_KEY.store(false, Ordering::SeqCst);
        }
        return 1; // Suppress the key press
    }

    if p_keyboard.vkCode == LEADER_KEY
        && (GetKeyState(VK_MENU as i32) as u16 & KEY_PRESSED_MASK) != 0
    {
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

        return 1; // Suppress the leader key press
    }

    CallNextHookEx(HOOK, n_code, w_param, l_param)
}

struct HotkeyManager;

impl HotkeyManager {
    fn new() -> Result<Self, &'static str> {
        unsafe {
            if RegisterHotKey(std::ptr::null_mut(), 1, MOD_ALT as u32, LEADER_KEY) == 0 {
                return Err("Failed to register hotkey.");
            }
        }
        Ok(Self)
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        unsafe {
            UnregisterHotKey(std::ptr::null_mut(), 1);
        }
    }
}

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

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    HotkeyManager::new()?;
    KeyboardHookManager::new()?;

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
