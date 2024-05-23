use std::ptr;

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

static mut HOOK_MANAGER: *const KeyboardHookManager = ptr::null();

const KEY_PRESSED_MASK: u16 = 0x8000;

pub struct KeyboardHookManager {
    hook: Option<HHOOK>,
    callback: Option<fn(u32, &Modifiers) -> bool>,
}

impl KeyboardHookManager {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            hook: None,
            callback: None,
        })
    }

    pub fn hook(&mut self, callback: fn(u32, &Modifiers) -> bool) -> Result<(), &'static str> {
        unsafe {
            if !HOOK_MANAGER.is_null() {
                return Err("Keyboard hook is already installed.");
            }

            HOOK_MANAGER = self;
            self.callback = Some(callback);

            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(Self::low_level_keyboard_proc),
                GetModuleHandleW(std::ptr::null()),
                0,
            );

            if hook.is_null() {
                return Err("Failed to install keyboard hook.");
            }

            self.hook = Some(hook);
            self.windows_loop();
            Ok(())
        }
    }

    fn windows_loop(&self) {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn get_instance() -> *const KeyboardHookManager {
        unsafe { HOOK_MANAGER }
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        let manager_ptr = Self::get_instance();
        if manager_ptr.is_null() {
            return 0;
        }

        let manager = &*manager_ptr;
        let hook = manager.hook.expect("No hook found!");
        let callback = manager.callback.expect("No callback found!");

        if n_code != 0 || (w_param != WM_KEYDOWN as WPARAM && w_param != WM_SYSKEYDOWN as WPARAM) {
            return CallNextHookEx(hook, n_code, w_param, l_param);
        }

        let p_keyboard: &KBDLLHOOKSTRUCT = &*(l_param as *const KBDLLHOOKSTRUCT);
        let modifiers = Modifiers {
            left_alt: (GetKeyState(VK_LMENU) as u16 & KEY_PRESSED_MASK) != 0,
        };

        if callback(p_keyboard.vkCode, &modifiers) {
            1
        } else {
            CallNextHookEx(hook, n_code, w_param, l_param)
        }
    }
}

impl Drop for KeyboardHookManager {
    fn drop(&mut self) {
        if let Some(hook) = self.hook {
            unsafe {
                UnhookWindowsHookEx(hook);
                HOOK_MANAGER = ptr::null();
            }
        }
    }
}

pub fn quit_windows_app() {
    unsafe {
        PostQuitMessage(0);
    }
}
