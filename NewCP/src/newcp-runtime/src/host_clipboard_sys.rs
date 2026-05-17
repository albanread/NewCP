//! `HostClipboardSys` native module — Win32 clipboard get/set.
//!
//! Backs `Mod/HostClipboardSys.cp` (DEFINITION module) and is consumed by
//! `Mod/HostClipboard.cp`.  Windows-only; a trivial stub that always returns
//! 0 (failure) is provided for other platforms.
//!
//! CP ABI:
//! - `IN text: ARRAY OF CHAR`  → `(*const u32, i64)` — UTF-32 wide string.
//! - `OUT text: ARRAY OF CHAR` → `(*mut u32, i64)` — UTF-32, capacity in i64.
//! - Return INTSHORT (i32): 1 = success, 0 = failure.

use crate::{ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact};

// ─── Platform implementation ─────────────────────────────────────────────────

#[cfg(windows)]
mod win {
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
    };
    use windows::Win32::System::Ole::CF_UNICODETEXT;

    /// Read UTF-32 CP string `(ptr, len)` into a UTF-16 Vec (NUL-terminated).
    fn utf32_to_utf16(ptr: *const u32, len: i64) -> Vec<u16> {
        if ptr.is_null() || len <= 0 {
            return vec![0u16];
        }
        let cap = len as usize;
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < cap {
            let cp = unsafe { *ptr.add(i) };
            if cp == 0 {
                break;
            }
            if let Some(c) = char::from_u32(cp) {
                let mut buf = [0u16; 2];
                let encoded = c.encode_utf16(&mut buf);
                for &u in encoded.iter() {
                    out.push(u);
                }
            } else {
                out.push(b'?' as u16);
            }
            i += 1;
        }
        out.push(0u16); // NUL terminator
        out
    }

    /// Write a NUL-terminated UTF-16 slice into a CP `OUT ARRAY OF CHAR`
    /// buffer (UTF-32, capacity `cap`).
    fn utf16_to_utf32_out(src: &[u16], dst_ptr: *mut u32, cap: usize) -> bool {
        if dst_ptr.is_null() || cap == 0 {
            return false;
        }
        let mut di = 0usize; // destination index (UTF-32 code-points written)
        let mut si = 0usize; // source index into src
        while si < src.len() {
            let u = src[si];
            if u == 0 {
                break;
            }
            // Decode UTF-16 surrogate pair if present
            let cp: u32 = if (0xD800..=0xDBFF).contains(&u) {
                si += 1;
                if si < src.len() {
                    let lo = src[si];
                    if (0xDC00..=0xDFFF).contains(&lo) {
                        0x10000u32 + (((u as u32 - 0xD800) << 10) | (lo as u32 - 0xDC00))
                    } else {
                        b'?' as u32
                    }
                } else {
                    b'?' as u32
                }
            } else {
                u as u32
            };
            if di + 1 >= cap {
                break; // leave room for NUL
            }
            unsafe { *dst_ptr.add(di) = cp };
            di += 1;
            si += 1;
        }
        unsafe { *dst_ptr.add(di) = 0 }; // NUL sentinel
        true
    }

    pub(super) fn get_text(text_ptr: *mut u32, text_len: i64) -> i32 {
        if text_ptr.is_null() || text_len <= 0 {
            return 0;
        }
        let cap = text_len as usize;

        if unsafe { OpenClipboard(None) }.is_err() {
            return 0;
        }

        let result = (|| {
            // GetClipboardData returns HANDLE (not HGLOBAL)
            let h: HANDLE = match unsafe { GetClipboardData(CF_UNICODETEXT.0 as u32) } {
                Ok(h) if !h.is_invalid() => h,
                _ => return 0i32,
            };

            // GlobalLock takes HGLOBAL — construct from the HANDLE's inner pointer
            let hglobal = HGLOBAL(h.0);
            let ptr = unsafe { GlobalLock(hglobal) };
            if ptr.is_null() {
                return 0;
            }
            let ptr16 = ptr as *const u16;

            // Measure NUL-terminated length
            let mut len16 = 0usize;
            while unsafe { *ptr16.add(len16) } != 0 {
                len16 += 1;
            }
            let slice16 = unsafe { std::slice::from_raw_parts(ptr16, len16 + 1) };
            let ok = utf16_to_utf32_out(slice16, text_ptr, cap);
            let _ = unsafe { GlobalUnlock(hglobal) };
            if ok { 1 } else { 0 }
        })();

        let _ = unsafe { CloseClipboard() };
        result
    }

    pub(super) fn set_text(text_ptr: *const u32, text_len: i64) -> i32 {
        let utf16 = utf32_to_utf16(text_ptr, text_len);
        let byte_size = utf16.len() * 2; // u16 units × 2 bytes each

        if unsafe { OpenClipboard(None) }.is_err() {
            return 0;
        }

        let result = (|| {
            if unsafe { EmptyClipboard() }.is_err() {
                return 0i32;
            }

            let hmem: HGLOBAL = match unsafe { GlobalAlloc(GMEM_MOVEABLE, byte_size) } {
                Ok(h) if !h.is_invalid() => h,
                _ => return 0,
            };

            let ptr = unsafe { GlobalLock(hmem) };
            if ptr.is_null() {
                let _ = unsafe { GlobalFree(Some(hmem)) };
                return 0;
            }
            let dst = ptr as *mut u16;
            for (i, &u) in utf16.iter().enumerate() {
                unsafe { *dst.add(i) = u };
            }
            let _ = unsafe { GlobalUnlock(hmem) };

            // SetClipboardData takes Option<HANDLE>; convert HGLOBAL → HANDLE
            let h_as_handle = HANDLE(hmem.0);
            match unsafe { SetClipboardData(CF_UNICODETEXT.0 as u32, Some(h_as_handle)) } {
                Ok(_) => 1,
                Err(_) => {
                    let _ = unsafe { GlobalFree(Some(hmem)) };
                    0
                }
            }
        })();

        let _ = unsafe { CloseClipboard() };
        result
    }
}

// ─── Exported C-ABI functions ─────────────────────────────────────────────────

/// `HostClipboardSys.GetText(OUT text: ARRAY OF CHAR): INTSHORT`
///
/// Returns 1 on success (text written), 0 on failure or empty clipboard.
#[unsafe(export_name = "HostClipboardSys.GetText")]
pub extern "C" fn host_clipboard_sys_get_text(
    text_ptr: *mut u32,
    text_len: i64,
) -> i32 {
    #[cfg(windows)]
    {
        win::get_text(text_ptr, text_len)
    }
    #[cfg(not(windows))]
    {
        if !text_ptr.is_null() && text_len > 0 {
            unsafe { *text_ptr = 0 };
        }
        0
    }
}

/// `HostClipboardSys.SetText(IN text: ARRAY OF CHAR): INTSHORT`
///
/// Returns 1 on success, 0 on failure.
#[unsafe(export_name = "HostClipboardSys.SetText")]
pub extern "C" fn host_clipboard_sys_set_text(
    text_ptr: *const u32,
    text_len: i64,
) -> i32 {
    #[cfg(windows)]
    {
        win::set_text(text_ptr, text_len)
    }
    #[cfg(not(windows))]
    {
        let _ = (text_ptr, text_len);
        0
    }
}

// ─── Native module registration ───────────────────────────────────────────────

pub fn native_module_artifact() -> NativeModuleArtifact {
    let names: &[(&str, *const ())] = &[
        ("GetText", host_clipboard_sys_get_text as *const ()),
        ("SetText", host_clipboard_sys_set_text as *const ()),
    ];
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostClipboardSys",
            vec![],
            ExportDirectory::new(
                names.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect(),
            ),
            "HostClipboardSys.bootstrap",
            "Rust-hosted Win32 clipboard facade for HostClipboard.cp",
            vec![],
        ),
        names
            .iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}
