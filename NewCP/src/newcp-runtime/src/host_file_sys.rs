//! Native `HostFileSys` module — flat C-ABI file I/O shims that back
//! `Mod/HostFiles.cp`'s concrete `Files` subclasses.
//!
//! The BlackBox `Host/Mod/Files.odc` calls Win32 directly (`CreateFileW`,
//! `ReadFile`, `WriteFile`, …). NewCP routes the same primitives through
//! `std::fs` so the file backend is portable. CP-side code calls the
//! exports below by name (`HostFileSys.Open`, `HostFileSys.ReadBytes`, …);
//! the JIT layer resolves them via `runtime_symbol_address`.
//!
//! ABI conventions:
//! - Paths are passed as null-terminated `*const u32` UTF-32 buffers
//!   (matching CP's `ARRAY OF CHAR`). Decoded to UTF-8 at the boundary.
//! - File handles are opaque `i64` values; `0` means "invalid handle".
//! - Byte buffers are `*mut u8` / `*const u8` plus an `i64` length, the
//!   same ABI used elsewhere for `ARRAY OF BYTE` parameters.
//! - All return values are `i64`. Error returns:
//!   - `Open`: returns `0` on failure.
//!   - `ReadBytes` / `WriteBytes`: returns the number of bytes actually
//!     transferred, or `-1` on error.
//!   - `Length` / `Pos`: returns `-1` on error.
//!   - `Delete` / `Exists` / `Rename`: returns `1` for true, `0` for false.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::{ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact};

/// Mode flag values matching `Files.cp` constants.
const MODE_READ: i64 = 0;
const MODE_WRITE: i64 = 1;
const MODE_READ_WRITE: i64 = 2;

struct FileSysState {
    next_handle: i64,
    files: HashMap<i64, File>,
}

impl FileSysState {
    fn new() -> Self {
        Self { next_handle: 1, files: HashMap::new() }
    }
}

static FILES: OnceLock<Mutex<FileSysState>> = OnceLock::new();

fn files() -> &'static Mutex<FileSysState> {
    FILES.get_or_init(|| Mutex::new(FileSysState::new()))
}

fn decode_utf32_path(ptr: *const u32) -> Option<PathBuf> {
    if ptr.is_null() {
        return None;
    }
    let mut s = String::new();
    let mut i = 0isize;
    loop {
        let cp = unsafe { *ptr.offset(i) };
        if cp == 0 {
            break;
        }
        s.push(char::from_u32(cp)?);
        i += 1;
        if i > 32_768 {
            // Defensive cap; CP `Files.Name` is 256, but we tolerate longer.
            return None;
        }
    }
    Some(PathBuf::from(s))
}

#[unsafe(export_name = "HostFileSys.Open")]
/// Open `path` (UTF-32, null-terminated) with `mode` (0=Read, 1=Write,
/// 2=ReadWrite). For modes that imply creation (Write, ReadWrite), the
/// file is created if it doesn't exist and truncated to zero length.
/// Returns the new handle, or `0` on failure.
///
/// `_path_len` is the open-array length passed by the CP fat-pointer
/// ABI for `IN path: ARRAY OF CHAR`; we ignore it and use the explicit
/// null terminator the CP path string carries.
pub extern "C" fn host_file_sys_open(path_ptr: *const u32, _path_len: i64, mode: i64) -> i64 {
    let Some(path) = decode_utf32_path(path_ptr) else { return 0 };

    let mut opts = OpenOptions::new();
    match mode {
        MODE_READ => { opts.read(true); }
        MODE_WRITE => { opts.write(true).create(true).truncate(true); }
        // CP `Directory.New` semantics: the file is *replaced*, not
        // appended-to. Truncate on open so a leftover file from a
        // previous run doesn't bleed bytes into the new write set.
        MODE_READ_WRITE => { opts.read(true).write(true).create(true).truncate(true); }
        _ => return 0,
    }

    let Ok(file) = opts.open(&path) else { return 0 };

    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let h = state.next_handle;
    state.next_handle = state.next_handle.wrapping_add(1);
    if state.next_handle == 0 { state.next_handle = 1; }
    state.files.insert(h, file);
    h
}

#[unsafe(export_name = "HostFileSys.Close")]
/// Close `handle`. No-op on invalid / already-closed handles.
pub extern "C" fn host_file_sys_close(handle: i64) {
    if handle == 0 { return; }
    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    state.files.remove(&handle);
}

#[unsafe(export_name = "HostFileSys.ReadBytes")]
/// Read up to `len` bytes from `handle` into `buf`. Returns the number
/// of bytes actually read (0 at EOF), or `-1` on error.
///
/// `_buf_len` is the CP open-array fat-pointer length for
/// `VAR buf: ARRAY OF BYTE`; we use the caller-supplied `len` to bound
/// the read instead so over-read by mistake is well-defined as `-1`.
pub extern "C" fn host_file_sys_read_bytes(
    handle: i64,
    buf: *mut u8,
    _buf_len: i64,
    len: i64,
) -> i64 {
    if handle == 0 || buf.is_null() || len <= 0 { return -1; }
    let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };

    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.read(slice) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

#[unsafe(export_name = "HostFileSys.WriteBytes")]
/// Write up to `len` bytes from `buf` to `handle`. Returns the number
/// of bytes actually written, or `-1` on error.
pub extern "C" fn host_file_sys_write_bytes(
    handle: i64,
    buf: *const u8,
    _buf_len: i64,
    len: i64,
) -> i64 {
    if handle == 0 || buf.is_null() || len <= 0 { return -1; }
    let slice = unsafe { std::slice::from_raw_parts(buf, len as usize) };

    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.write(slice) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

#[unsafe(export_name = "HostFileSys.ReadByte")]
/// Read a single byte from `handle`. Returns the byte (0..255) or `-1`
/// at EOF / on error.
pub extern "C" fn host_file_sys_read_byte(handle: i64) -> i64 {
    if handle == 0 { return -1; }
    let mut buf = [0u8; 1];
    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.read(&mut buf) {
        Ok(1) => buf[0] as i64,
        _ => -1,
    }
}

#[unsafe(export_name = "HostFileSys.WriteByte")]
/// Write a single byte to `handle`. Returns `1` on success, `-1` on error.
pub extern "C" fn host_file_sys_write_byte(handle: i64, byte: i64) -> i64 {
    if handle == 0 { return -1; }
    let buf = [(byte & 0xFF) as u8];
    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.write(&buf) {
        Ok(1) => 1,
        _ => -1,
    }
}

#[unsafe(export_name = "HostFileSys.Length")]
/// Return the total length of the file in bytes, or `-1` on error.
pub extern "C" fn host_file_sys_length(handle: i64) -> i64 {
    if handle == 0 { return -1; }
    let state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get(&handle) else { return -1; };
    match file.metadata() {
        Ok(md) => md.len() as i64,
        Err(_) => -1,
    }
}

#[unsafe(export_name = "HostFileSys.Pos")]
/// Return the current read/write position in bytes, or `-1` on error.
pub extern "C" fn host_file_sys_pos(handle: i64) -> i64 {
    if handle == 0 { return -1; }
    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.stream_position() {
        Ok(p) => p as i64,
        Err(_) => -1,
    }
}

#[unsafe(export_name = "HostFileSys.SetPos")]
/// Seek to byte offset `pos` (from the start of the file). Returns `1`
/// on success, `-1` on error.
pub extern "C" fn host_file_sys_set_pos(handle: i64, pos: i64) -> i64 {
    if handle == 0 || pos < 0 { return -1; }
    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.seek(SeekFrom::Start(pos as u64)) {
        Ok(_) => 1,
        Err(_) => -1,
    }
}

#[unsafe(export_name = "HostFileSys.Flush")]
/// Flush buffered writes for `handle`. Returns `1` on success, `-1` on error.
pub extern "C" fn host_file_sys_flush(handle: i64) -> i64 {
    if handle == 0 { return -1; }
    let mut state = files().lock().expect("HostFileSys mutex poisoned");
    let Some(file) = state.files.get_mut(&handle) else { return -1; };
    match file.flush() {
        Ok(_) => 1,
        Err(_) => -1,
    }
}

#[unsafe(export_name = "HostFileSys.Exists")]
/// Returns `1` if `path` exists in the filesystem, `0` otherwise.
pub extern "C" fn host_file_sys_exists(path_ptr: *const u32, _path_len: i64) -> i64 {
    let Some(path) = decode_utf32_path(path_ptr) else { return 0 };
    if path.exists() { 1 } else { 0 }
}

#[unsafe(export_name = "HostFileSys.Delete")]
/// Delete `path`. Returns `1` on success, `0` on failure (incl. not present).
pub extern "C" fn host_file_sys_delete(path_ptr: *const u32, _path_len: i64) -> i64 {
    let Some(path) = decode_utf32_path(path_ptr) else { return 0 };
    match std::fs::remove_file(&path) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[unsafe(export_name = "HostFileSys.Rename")]
/// Rename `old` to `new`. Returns `1` on success, `0` on failure.
pub extern "C" fn host_file_sys_rename(
    old_ptr: *const u32,
    _old_len: i64,
    new_ptr: *const u32,
    _new_len: i64,
) -> i64 {
    let Some(old_path) = decode_utf32_path(old_ptr) else { return 0 };
    let Some(new_path) = decode_utf32_path(new_ptr) else { return 0 };
    match std::fs::rename(&old_path, &new_path) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

// -- Native module registration ------------------------------------------

pub fn native_module_artifact() -> NativeModuleArtifact {
    let names: &[(&str, *const ())] = &[
        ("Open",       host_file_sys_open       as *const ()),
        ("Close",      host_file_sys_close      as *const ()),
        ("ReadBytes",  host_file_sys_read_bytes as *const ()),
        ("WriteBytes", host_file_sys_write_bytes as *const ()),
        ("ReadByte",   host_file_sys_read_byte  as *const ()),
        ("WriteByte",  host_file_sys_write_byte as *const ()),
        ("Length",     host_file_sys_length     as *const ()),
        ("Pos",        host_file_sys_pos        as *const ()),
        ("SetPos",     host_file_sys_set_pos    as *const ()),
        ("Flush",      host_file_sys_flush      as *const ()),
        ("Exists",     host_file_sys_exists     as *const ()),
        ("Delete",     host_file_sys_delete     as *const ()),
        ("Rename",     host_file_sys_rename     as *const ()),
    ];
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostFileSys",
            vec![],
            ExportDirectory::new(
                names.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect(),
            ),
            "HostFileSys.bootstrap",
            "Rust-hosted std::fs file I/O facade for HostFiles.cp",
            vec![],
        ),
        names.iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf32(s: &str) -> Vec<u32> {
        let mut v: Vec<u32> = s.chars().map(|c| c as u32).collect();
        v.push(0);
        v
    }

    #[test]
    fn round_trip_write_then_read() {
        let dir = std::env::temp_dir();
        let path = dir.join("newcp_hostfilesys_roundtrip.bin");
        let path_str = path.to_string_lossy().to_string();
        let path_buf = utf32(&path_str);

        let path_len = path_buf.len() as i64;
        let h = host_file_sys_open(path_buf.as_ptr(), path_len, MODE_WRITE);
        assert_ne!(h, 0, "open for write should succeed");
        let payload: [u8; 5] = [1, 2, 3, 4, 5];
        let written = host_file_sys_write_bytes(h, payload.as_ptr(), payload.len() as i64, payload.len() as i64);
        assert_eq!(written, 5);
        host_file_sys_close(h);

        let h2 = host_file_sys_open(path_buf.as_ptr(), path_len, MODE_READ);
        assert_ne!(h2, 0);
        let len = host_file_sys_length(h2);
        assert_eq!(len, 5);
        let mut buf = [0u8; 5];
        let read = host_file_sys_read_bytes(h2, buf.as_mut_ptr(), buf.len() as i64, buf.len() as i64);
        assert_eq!(read, 5);
        assert_eq!(buf, payload);
        host_file_sys_close(h2);

        host_file_sys_delete(path_buf.as_ptr(), path_len);
    }
}
