//! The clipboard, for passing drawings between players as text. The tests use a stand-in, so
//! running them never touches what the person at the machine has copied.

/// The most text read back from the clipboard, in UTF-16 units: a clipboard can hold anything.
#[cfg(not(test))]
const MAX_UNITS: usize = 1 << 20;

#[cfg(not(test))]
mod os {
    use super::MAX_UNITS;
    use winapi::um::winbase::{GlobalAlloc, GlobalFree, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
    use winapi::um::winuser::{CloseClipboard, EmptyClipboard, GetActiveWindow, GetClipboardData, OpenClipboard, SetClipboardData, CF_UNICODETEXT};

    pub fn set(text: &str) -> bool {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            if OpenClipboard(GetActiveWindow()) == 0 {
                return false;
            }
            EmptyClipboard();
            let h = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);
            let mut ok = false;
            if !h.is_null() {
                let p = GlobalLock(h) as *mut u16;
                if !p.is_null() {
                    std::ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
                    GlobalUnlock(h);
                    // Once the clipboard has it, the memory is the clipboard's.
                    ok = !SetClipboardData(CF_UNICODETEXT, h).is_null();
                }
                if !ok {
                    GlobalFree(h);
                }
            }
            CloseClipboard();
            ok
        }
    }

    pub fn get() -> Option<String> {
        unsafe {
            if OpenClipboard(GetActiveWindow()) == 0 {
                return None;
            }
            let h = GetClipboardData(CF_UNICODETEXT);
            let mut out = None;
            if !h.is_null() {
                let p = GlobalLock(h) as *const u16;
                if !p.is_null() {
                    // Up to its end mark, and never past the memory it was given or the cap.
                    let room = (GlobalSize(h) / 2).min(MAX_UNITS);
                    let mut n = 0;
                    while n < room && *p.add(n) != 0 {
                        n += 1;
                    }
                    out = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, n)));
                    GlobalUnlock(h);
                }
            }
            CloseClipboard();
            out
        }
    }
}

#[cfg(test)]
mod os {
    use std::cell::RefCell;
    thread_local!(static HELD: RefCell<Option<String>> = const { RefCell::new(None) });

    pub fn set(text: &str) -> bool {
        HELD.with(|h| *h.borrow_mut() = Some(text.to_string()));
        true
    }

    pub fn get() -> Option<String> {
        HELD.with(|h| h.borrow().clone())
    }
}

pub use os::{get, set};
