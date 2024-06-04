use crate::types::Event;
use crate::types::Modifier;
use crate::types::SystemAction::*;
use std::fmt::Debug;
use std::fmt::Display;
use std::ptr;
use std::sync::mpsc;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetKeyState, GetMessageW, PostQuitMessage, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_SYSKEYDOWN,
};

use winapi::um::winuser::VK_LMENU;

static mut HOOK_MANAGER: *mut KeyboardHookManager = ptr::null_mut();

const KEY_PRESSED_MASK: u16 = 0x8000;

pub(crate) enum HookAction {
    Suppress,
    PassOn,
}

pub(crate) trait KeypressCallback {
    fn handle(&mut self, key: u32, modifiers: &[Modifier]) -> HookAction;
}

type BoxedKeypressCallback = Box<dyn KeypressCallback>;

pub(crate) struct KeyboardHookManager {
    hook: Option<HHOOK>,
    callback: Option<BoxedKeypressCallback>,
}

impl KeyboardHookManager {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            hook: None,
            callback: None,
        })
    }

    pub fn hook<A, T>(
        &mut self,
        sender: mpsc::Sender<Event<A, T>>,
        keypress_callback: BoxedKeypressCallback,
    ) -> Result<(), &'static str>
    where
        A: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
        T: PartialEq + Eq + Clone + Debug + Display + Sync + Send,
    {
        unsafe {
            if !HOOK_MANAGER.is_null() {
                return Err("Keyboard hook is already installed.");
            }

            HOOK_MANAGER = self;
            self.callback = Some(keypress_callback);

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
            sender.send(Event::System(Hello)).unwrap();
            Self::start_windows_loop();
            Ok(())
        }
    }

    pub fn stop_windows_loop() {
        unsafe {
            PostQuitMessage(0);
        }
    }

    fn start_windows_loop() {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn get_instance() -> *mut KeyboardHookManager {
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

        let manager = &mut *manager_ptr;
        let hook = manager.hook.expect("No hook found!");
        let callback: &mut Box<dyn KeypressCallback> =
            manager.callback.as_mut().expect("No callback found!");

        if n_code != 0 || (w_param != WM_KEYDOWN as WPARAM && w_param != WM_SYSKEYDOWN as WPARAM) {
            return CallNextHookEx(hook, n_code, w_param, l_param);
        }

        let p_keyboard: &KBDLLHOOKSTRUCT = &*(l_param as *const KBDLLHOOKSTRUCT);
        let modifiers = if (GetKeyState(VK_LMENU) as u16 & KEY_PRESSED_MASK) != 0 {
            vec![Modifier::ModAlt]
        } else {
            vec![]
        };

        match callback.handle(p_keyboard.vkCode, &modifiers) {
            HookAction::Suppress => 1,
            HookAction::PassOn => CallNextHookEx(hook, n_code, w_param, l_param),
        }
    }
}

impl Drop for KeyboardHookManager {
    fn drop(&mut self) {
        if let Some(hook) = self.hook {
            unsafe {
                UnhookWindowsHookEx(hook);
                HOOK_MANAGER = ptr::null_mut();
            }
        }
    }
}
