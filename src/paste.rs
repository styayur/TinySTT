//! Optional automatic paste into the foreground window.

use crate::error::Result;

#[cfg(all(windows, feature = "native"))]
pub fn send_ctrl_v() -> Result<()> {
    use std::thread;
    use std::time::Duration;

    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        keybd_event, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    };

    // Give Windows a moment to publish the clipboard before key injection.
    thread::sleep(Duration::from_millis(35));
    unsafe {
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(VK_V as u8, 0, 0, 0);
        keybd_event(VK_V as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    Ok(())
}

#[cfg(not(all(windows, feature = "native")))]
pub fn send_ctrl_v() -> Result<()> {
    Err(crate::error::TinySttError::PasteFailed(
        "automatic paste is only supported on Windows".to_string(),
    ))
}
