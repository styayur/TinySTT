//! Windows clipboard text output with bounded retry for clipboard contention.

use crate::error::Result;

#[cfg(all(windows, feature = "native"))]
pub fn write_text(text: &str) -> Result<()> {
    use std::ptr;
    use std::thread;
    use std::time::Duration;

    use windows_sys::Win32::Foundation::GlobalFree;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };
    use windows_sys::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
    };

    const CF_UNICODETEXT: u32 = 13;

    let mut opened = false;
    for _ in 0..10 {
        if unsafe { OpenClipboard(ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    if !opened {
        return Err(crate::error::TinySttError::ClipboardUnavailable(
            "OpenClipboard failed after retries".to_string(),
        ));
    }

    let result = (|| -> Result<()> {
        if unsafe { EmptyClipboard() } == 0 {
            return Err(crate::error::TinySttError::ClipboardUnavailable(
                "EmptyClipboard failed".to_string(),
            ));
        }

        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * std::mem::size_of::<u16>();
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) };
        if handle.is_null() {
            return Err(crate::error::TinySttError::ClipboardUnavailable(
                "GlobalAlloc failed".to_string(),
            ));
        }

        let locked = unsafe { GlobalLock(handle) };
        if locked.is_null() {
            unsafe {
                let _ = GlobalFree(handle);
            }
            return Err(crate::error::TinySttError::ClipboardUnavailable(
                "GlobalLock failed".to_string(),
            ));
        }
        unsafe {
            ptr::copy_nonoverlapping(wide.as_ptr(), locked as *mut u16, wide.len());
            let _ = GlobalUnlock(handle);
        }

        if unsafe { SetClipboardData(CF_UNICODETEXT, handle) }.is_null() {
            unsafe {
                let _ = GlobalFree(handle);
            }
            return Err(crate::error::TinySttError::ClipboardUnavailable(
                "SetClipboardData failed".to_string(),
            ));
        }
        Ok(())
    })();

    unsafe {
        let _ = CloseClipboard();
    }
    result
}

#[cfg(not(all(windows, feature = "native")))]
pub fn write_text(_text: &str) -> Result<()> {
    Err(crate::error::TinySttError::ClipboardUnavailable(
        "clipboard is only supported on Windows".to_string(),
    ))
}
