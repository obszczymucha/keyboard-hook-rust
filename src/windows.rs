use crate::handle_key_press;
use crate::types::Modifiers;

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

const KEY_PRESSED_MASK: u16 = 0x8000;

static mut HOOK: HHOOK = std::ptr::null_mut();
pub struct KeyboardHookManager;

impl KeyboardHookManager {
    pub fn new() -> Result<Self, &'static str> {
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
        left_alt: (GetKeyState(VK_LMENU) as u16 & KEY_PRESSED_MASK) != 0,
    };

    if handle_key_press(p_keyboard.vkCode, &modifiers) == true {
        return 1;
    } else {
        return CallNextHookEx(HOOK, n_code, w_param, l_param);
    }
}

pub fn windows_loop() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

pub fn quit_windows_app() {
    unsafe {
        PostQuitMessage(0);
    }
}
