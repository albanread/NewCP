//! `HostDialogSys` native module — Win32 file-open / file-save dialogs.
//!
//! Backs `Mod/HostDialogSys.cp` (DEFINITION module) and is consumed by
//! `Mod/HostDialog.cp`.  The module is Windows-only; a trivial stub that
//! always returns 0 (cancelled) is provided for other platforms so the
//! CP layer still links, but in practice it can't be reached because the
//! igui module also requires Windows.
//!
//! CP ABI notes:
//! - `IN s: ARRAY OF SHORTCHAR` → `(ptr: *const u8, len: i64)` fat ptr.
//!   The string is NUL-terminated; `len` is the array capacity.
//! - `IN s: ARRAY OF CHAR`     → `(ptr: *const u32, len: i64)`.
//! - `OUT s: ARRAY OF CHAR`    → `(ptr: *mut u32, len: i64)`.
//!   `len` is the declared capacity; we write up to `len-1` code-points
//!   and always append a NUL terminator.

use crate::{ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact};

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Write a Rust `str` (UTF-8) into a CP `OUT path: ARRAY OF CHAR` buffer
/// (UTF-32, capacity = `path_len` code-points).
/// Returns the number of code-points written (excluding the NUL sentinel).
fn write_utf32_out(s: &str, path_ptr: *mut u32, path_len: i64) -> usize {
    if path_ptr.is_null() || path_len <= 0 {
        return 0;
    }
    let cap = path_len as usize;
    let mut i = 0usize;
    for ch in s.chars() {
        if i + 1 >= cap {
            break; // leave room for NUL
        }
        unsafe { *path_ptr.add(i) = ch as u32 };
        i += 1;
    }
    unsafe { *path_ptr.add(i) = 0 }; // NUL sentinel
    i
}

// ─── Platform implementations ───────────────────────────────────────────────

#[cfg(windows)]
mod win {
    use super::write_utf32_out;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Controls::Dialogs::{
        GetOpenFileNameW, GetSaveFileNameW, OPENFILENAMEW, OFN_EXPLORER, OFN_FILEMUSTEXIST,
        OFN_HIDEREADONLY, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST,
    };
    use windows::core::PCWSTR;

    /// Get the main frame HWND for use as dialog parent; falls back to
    /// NULL (desktop parent) if the frame hasn't been created yet.
    fn owner_hwnd() -> HWND {
        crate::igui::cp_exports::FRAME_HWND
            .get()
            .map(|&raw| HWND(raw as *mut _))
            .unwrap_or_default()
    }

    /// Decode a NUL-terminated `ARRAY OF SHORTCHAR` (CP ABI) into a
    /// UTF-16 filter string suitable for `OPENFILENAMEW.lpstrFilter`.
    /// The filter is a sequence of NUL-separated description+pattern
    /// pairs terminated by a double-NUL:
    ///   "Text Files\0*.txt\0All Files\0*.*\0\0"
    fn shortchar_to_filter_utf16(ptr: *const u8, _len: i64) -> Vec<u16> {
        if ptr.is_null() {
            // Default: accept all files.
            return "All Files\0*.*\0\0".encode_utf16().collect();
        }
        // Copy raw bytes until we see two consecutive NULs (end of filter).
        let mut bytes: Vec<u8> = Vec::new();
        let mut i = 0isize;
        loop {
            let b = unsafe { *ptr.offset(i) };
            bytes.push(b);
            i += 1;
            // Detect the double-NUL terminator.
            if bytes.len() >= 2 {
                let n = bytes.len();
                if bytes[n - 1] == 0 && bytes[n - 2] == 0 {
                    break;
                }
            }
            if i > 4096 {
                bytes.push(0); // ensure termination
                break;
            }
        }
        // Convert bytes (Latin-1) to UTF-16.
        bytes.iter().map(|&b| b as u16).collect()
    }

    /// Decode a NUL-terminated `ARRAY OF CHAR` (UTF-32 CP ABI) into a
    /// UTF-16 string (for populating the initial filename in Save dialog).
    fn char_to_utf16(ptr: *const u32, _len: i64) -> Vec<u16> {
        if ptr.is_null() {
            return vec![0u16];
        }
        let mut out: Vec<u16> = Vec::new();
        let mut i = 0isize;
        loop {
            let cp = unsafe { *ptr.offset(i) };
            if cp == 0 || i > 1024 {
                break;
            }
            if let Some(ch) = char::from_u32(cp) {
                let mut tmp = [0u16; 2];
                let enc = ch.encode_utf16(&mut tmp);
                out.extend_from_slice(enc);
            }
            i += 1;
        }
        out.push(0u16); // NUL terminate
        out
    }

    pub fn show_open_file(
        filter_ptr: *const u8,
        filter_len: i64,
        path_ptr: *mut u32,
        path_len: i64,
    ) -> i32 {
        let owner = owner_hwnd();
        let filter = shortchar_to_filter_utf16(filter_ptr, filter_len);
        let mut buf = vec![0u16; 1024];

        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: owner,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            nFilterIndex: 1,
            lpstrFile: windows::core::PWSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            Flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY,
            ..Default::default()
        };

        let ok = unsafe { GetOpenFileNameW(&mut ofn) }.as_bool();
        if !ok {
            if !path_ptr.is_null() && path_len > 0 {
                unsafe { *path_ptr = 0 };
            }
            return 0;
        }

        // Find the NUL in the result buffer and decode as a UTF-8 path.
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let path_str = OsString::from_wide(&buf[..n])
            .to_string_lossy()
            .into_owned();
        write_utf32_out(&path_str, path_ptr, path_len);
        1
    }

    pub fn show_save_file(
        filter_ptr: *const u8,
        filter_len: i64,
        initial_ptr: *const u32,
        initial_len: i64,
        path_ptr: *mut u32,
        path_len: i64,
    ) -> i32 {
        let owner = owner_hwnd();
        let filter = shortchar_to_filter_utf16(filter_ptr, filter_len);
        // Pre-populate the filename buffer with the suggested name.
        let initial = char_to_utf16(initial_ptr, initial_len);
        let mut buf = vec![0u16; 1024];
        let copy_n = initial.len().min(buf.len() - 1);
        buf[..copy_n].copy_from_slice(&initial[..copy_n]);

        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: owner,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            nFilterIndex: 1,
            lpstrFile: windows::core::PWSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            Flags: OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_OVERWRITEPROMPT,
            ..Default::default()
        };

        let ok = unsafe { GetSaveFileNameW(&mut ofn) }.as_bool();
        if !ok {
            if !path_ptr.is_null() && path_len > 0 {
                unsafe { *path_ptr = 0 };
            }
            return 0;
        }

        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let path_str = OsString::from_wide(&buf[..n])
            .to_string_lossy()
            .into_owned();
        write_utf32_out(&path_str, path_ptr, path_len);
        1
    }
}

// ─── Exported C-ABI functions ────────────────────────────────────────────────

/// `HostDialogSys.ShowOpenFile(IN filter: ARRAY OF SHORTCHAR;
///                              OUT path: ARRAY OF CHAR): INTSHORT`
///
/// Shows a Windows Open-File dialog.
/// Returns 1 if the user picked a file, 0 if cancelled.
/// On success `path` receives the chosen file path (UTF-32, NUL-terminated).
#[unsafe(export_name = "HostDialogSys.ShowOpenFile")]
pub extern "C" fn host_dialog_sys_show_open_file(
    filter_ptr: *const u8,
    filter_len: i64,
    path_ptr: *mut u32,
    path_len: i64,
) -> i32 {
    #[cfg(windows)]
    {
        win::show_open_file(filter_ptr, filter_len, path_ptr, path_len)
    }
    #[cfg(not(windows))]
    {
        let _ = (filter_ptr, filter_len);
        if !path_ptr.is_null() && path_len > 0 {
            unsafe { *path_ptr = 0 };
        }
        0
    }
}

/// `HostDialogSys.ShowSaveFile(IN filter: ARRAY OF SHORTCHAR;
///                              IN initialName: ARRAY OF CHAR;
///                              OUT path: ARRAY OF CHAR): INTSHORT`
///
/// Shows a Windows Save-File dialog pre-populated with `initialName`.
/// Returns 1 if the user confirmed a save path, 0 if cancelled.
#[unsafe(export_name = "HostDialogSys.ShowSaveFile")]
pub extern "C" fn host_dialog_sys_show_save_file(
    filter_ptr: *const u8,
    filter_len: i64,
    initial_ptr: *const u32,
    initial_len: i64,
    path_ptr: *mut u32,
    path_len: i64,
) -> i32 {
    #[cfg(windows)]
    {
        win::show_save_file(filter_ptr, filter_len, initial_ptr, initial_len, path_ptr, path_len)
    }
    #[cfg(not(windows))]
    {
        let _ = (filter_ptr, filter_len, initial_ptr, initial_len);
        if !path_ptr.is_null() && path_len > 0 {
            unsafe { *path_ptr = 0 };
        }
        0
    }
}

// ─── Native module registration ──────────────────────────────────────────────

pub fn native_module_artifact() -> NativeModuleArtifact {
    let names: &[(&str, *const ())] = &[
        ("ShowOpenFile", host_dialog_sys_show_open_file as *const ()),
        ("ShowSaveFile", host_dialog_sys_show_save_file as *const ()),
    ];
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostDialogSys",
            vec![],
            ExportDirectory::new(
                names.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect(),
            ),
            "HostDialogSys.bootstrap",
            "Rust-hosted Win32 file-dialog facade for HostDialog.cp",
            vec![],
        ),
        names
            .iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}
