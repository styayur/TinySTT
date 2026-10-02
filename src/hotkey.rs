//! Global push-to-talk hotkey.
//!
//! Windows `RegisterHotKey` cannot report key release, so TinySTT uses a
//! low-level keyboard hook and emits separate Pressed / Released events.

use crate::error::{Result, TinySttError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeySpec {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub key: String,
    pub virtual_key: u16,
    pub display: String,
}

impl HotkeySpec {
    pub fn parse(input: &str) -> Result<Self> {
        let original = input.trim();
        if original.is_empty() {
            return Err(TinySttError::HotkeyRegistration(
                "hotkey cannot be empty".to_string(),
            ));
        }

        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut win = false;
        let mut key = None;

        for part in original.split('+').map(str::trim) {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "alt" => alt = true,
                "shift" => shift = true,
                "win" | "windows" | "super" => win = true,
                "" => {
                    return Err(TinySttError::HotkeyRegistration(format!(
                        "invalid hotkey syntax: {original}"
                    )))
                }
                _ => {
                    if key.replace(part.to_ascii_uppercase()).is_some() {
                        return Err(TinySttError::HotkeyRegistration(format!(
                            "hotkey must contain exactly one non-modifier key: {original}"
                        )));
                    }
                }
            }
        }

        if !(ctrl || alt || shift || win) {
            return Err(TinySttError::HotkeyRegistration(
                "hotkey must include Ctrl, Alt, Shift, or Win".to_string(),
            ));
        }
        let key = key.ok_or_else(|| {
            TinySttError::HotkeyRegistration(format!("hotkey has no key: {original}"))
        })?;
        let virtual_key = virtual_key_for(&key).ok_or_else(|| {
            TinySttError::HotkeyRegistration(format!(
                "unsupported hotkey key '{key}'; use A-Z, 0-9, or F1-F24"
            ))
        })?;

        let mut parts = Vec::new();
        if ctrl {
            parts.push("Ctrl");
        }
        if alt {
            parts.push("Alt");
        }
        if shift {
            parts.push("Shift");
        }
        if win {
            parts.push("Win");
        }
        let display = format!("{}+{}", parts.join("+"), key);

        Ok(Self {
            ctrl,
            alt,
            shift,
            win,
            key,
            virtual_key,
            display,
        })
    }
}

fn virtual_key_for(key: &str) -> Option<u16> {
    let bytes = key.as_bytes();
    if bytes.len() == 1 {
        let ch = bytes[0];
        if ch.is_ascii_uppercase() || ch.is_ascii_digit() {
            return Some(ch as u16);
        }
    }
    if let Some(number) = key.strip_prefix('F') {
        let number: u16 = number.parse().ok()?;
        if (1..=24).contains(&number) {
            return Some(0x70 + number - 1);
        }
    }
    None
}

#[cfg(all(windows, feature = "native"))]
mod platform {
    use super::{HotkeyEvent, HotkeySpec};
    use crate::error::{Result, TinySttError};
    use std::ptr;
    use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
    use std::sync::{Mutex, OnceLock};
    use std::thread;
    use std::time::Duration;

    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, LLKHF_INJECTED, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    struct HookRuntime {
        tx: Sender<HotkeyEvent>,
        spec: HotkeySpec,
        action_down: bool,
    }

    static HOOK_RUNTIME: OnceLock<Mutex<HookRuntime>> = OnceLock::new();

    pub struct GlobalHotkey {
        rx: Receiver<HotkeyEvent>,
        thread_id: u32,
    }

    impl GlobalHotkey {
        pub fn register(spec: HotkeySpec) -> Result<Self> {
            let (event_tx, event_rx) = mpsc::channel();
            HOOK_RUNTIME
                .set(Mutex::new(HookRuntime {
                    tx: event_tx,
                    spec,
                    action_down: false,
                }))
                .map_err(|_| {
                    TinySttError::HotkeyRegistration(
                        "a global hotkey is already registered".to_string(),
                    )
                })?;

            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            thread::Builder::new()
                .name("tinystt-hotkey".to_string())
                .spawn(move || run_hook_thread(ready_tx))
                .map_err(|e| TinySttError::HotkeyRegistration(e.to_string()))?;

            let thread_id = ready_rx.recv_timeout(Duration::from_secs(5)).map_err(|_| {
                TinySttError::HotkeyRegistration(
                    "timed out while starting the keyboard hook".to_string(),
                )
            })?;
            if thread_id == 0 {
                return Err(TinySttError::HotkeyRegistration(
                    "SetWindowsHookExW failed".to_string(),
                ));
            }

            Ok(Self {
                rx: event_rx,
                thread_id,
            })
        }

        pub fn try_recv(&self) -> Option<HotkeyEvent> {
            self.rx.try_recv().ok()
        }
    }

    impl Drop for GlobalHotkey {
        fn drop(&mut self) {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
            }
        }
    }

    fn run_hook_thread(ready: SyncSender<u32>) {
        unsafe {
            let hmod = GetModuleHandleW(ptr::null());
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0);
            if hook.is_null() {
                let _ = ready.send(0);
                return;
            }
            let thread_id = GetCurrentThreadId();
            let _ = ready.send(thread_id);

            let mut message = std::mem::zeroed();
            while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            let _ = UnhookWindowsHookEx(hook);
        }
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 && lparam != 0 {
            let event = &*(lparam as *const KBDLLHOOKSTRUCT);
            if event.flags & LLKHF_INJECTED == 0 {
                let message = wparam as u32;
                if let Some(runtime) = HOOK_RUNTIME.get() {
                    if let Ok(mut runtime) = runtime.lock() {
                        let spec = runtime.spec.clone();
                        let key = event.vkCode as u16;
                        let key_down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
                        let key_up = message == WM_KEYUP || message == WM_SYSKEYUP;

                        if key == spec.virtual_key
                            && key_down
                            && modifiers_match(&spec)
                            && !runtime.action_down
                        {
                            runtime.action_down = true;
                            let _ = runtime.tx.send(HotkeyEvent::Pressed);
                        } else if runtime.action_down
                            && key_up
                            && (key == spec.virtual_key || !modifiers_match(&spec))
                        {
                            runtime.action_down = false;
                            let _ = runtime.tx.send(HotkeyEvent::Released);
                        }
                    }
                }
            }
        }
        CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
    }
    fn modifiers_match(spec: &HotkeySpec) -> bool {
        is_down(VK_CONTROL) == spec.ctrl
            && is_down(VK_MENU) == spec.alt
            && is_down(VK_SHIFT) == spec.shift
            && (is_down(VK_LWIN) || is_down(VK_RWIN)) == spec.win
    }

    fn is_down(key: u16) -> bool {
        unsafe { GetAsyncKeyState(key as i32) & 0x8000u16 as i16 != 0 }
    }
}

#[cfg(not(all(windows, feature = "native")))]
mod platform {
    use super::{HotkeyEvent, HotkeySpec};
    use crate::error::{Result, TinySttError};

    pub struct GlobalHotkey;

    impl GlobalHotkey {
        pub fn register(_spec: HotkeySpec) -> Result<Self> {
            Err(TinySttError::HotkeyRegistration(
                "global push-to-talk is only supported on Windows".to_string(),
            ))
        }

        pub fn try_recv(&self) -> Option<HotkeyEvent> {
            None
        }
    }
}

pub use platform::GlobalHotkey;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_hotkey() {
        let spec = HotkeySpec::parse("Ctrl+Alt+D").unwrap();
        assert_eq!(spec.display, "Ctrl+Alt+D");
        assert_eq!(spec.virtual_key, b'D' as u16);
    }

    #[test]
    fn rejects_modifier_only_hotkeys() {
        assert!(HotkeySpec::parse("Ctrl+Alt").is_err());
    }
}
