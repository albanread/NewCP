use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::Mutex;

use crate::{ExportDirectory, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact};

struct ConsoleState {
    capture_active: bool,
    input: VecDeque<u8>,
    output: String,
    write_callback: Option<Box<dyn Fn(&str) + Send + 'static>>,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            capture_active: false,
            input: VecDeque::new(),
            output: String::new(),
            write_callback: None,
        }
    }
}

static CONSOLE_STATE: Mutex<ConsoleState> = Mutex::new(ConsoleState {
    capture_active: false,
    input: VecDeque::new(),
    output: String::new(),
    write_callback: None,
});

pub fn begin_capture() {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.capture_active = true;
    state.output.clear();
}

pub fn end_capture() -> String {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.capture_active = false;
    std::mem::take(&mut state.output)
}

pub fn set_input(input: &str) {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.input = input.bytes().collect();
}

/// Feed a raw byte sequence into the console input buffer (test helper).
/// Use this when the input contains multi-byte UTF-8 sequences that can't
/// be expressed as Rust string literal `\x` escapes (which cap at 0x7F).
#[cfg(test)]
pub fn set_input_bytes(input: &[u8]) {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.input = input.iter().copied().collect();
}

pub fn reset() {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.capture_active = false;
    state.input.clear();
    state.output.clear();
    state.write_callback = None;
}

/// Install a callback that is invoked (under the console lock) on every write.
/// Replaces any previously installed callback.
pub fn set_write_callback(f: impl Fn(&str) + Send + 'static) {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.write_callback = Some(Box::new(f));
}

/// Remove any installed write callback.
pub fn clear_write_callback() {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    state.write_callback = None;
}

fn write_text(text: &str) {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    if state.capture_active {
        state.output.push_str(text);
        return;
    }
    if let Some(cb) = &state.write_callback {
        cb(text);
        return;
    }
    drop(state);

    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
}

fn read_byte() -> Option<u8> {
    let mut state = CONSOLE_STATE.lock().expect("console mutex poisoned");
    if let Some(byte) = state.input.pop_front() {
        return Some(byte);
    }
    drop(state);

    let mut stdin = std::io::stdin().lock();
    let mut buf = [0u8; 1];
    match stdin.read(&mut buf) {
        Ok(1) => Some(buf[0]),
        _ => None,
    }
}

fn read_non_whitespace_byte() -> Option<u8> {
    loop {
        let byte = read_byte()?;
        if !(byte as char).is_whitespace() {
            return Some(byte);
        }
    }
}

/// Decode one UTF-8 scalar value from the byte stream.
///
/// Reads 1–4 bytes as needed. Returns `'\u{FFFD}'` on any encoding error
/// rather than propagating garbage code points to the caller.
fn read_utf8_codepoint() -> Option<char> {
    let b0 = read_byte()?;
    // ASCII fast path.
    if b0 < 0x80 {
        return char::from_u32(b0 as u32);
    }
    // Determine continuation-byte count and seed bits from the leading byte.
    let (n_cont, seed) = if b0 & 0xE0 == 0xC0 {
        (1u8, (b0 & 0x1F) as u32)
    } else if b0 & 0xF0 == 0xE0 {
        (2, (b0 & 0x0F) as u32)
    } else if b0 & 0xF8 == 0xF0 {
        (3, (b0 & 0x07) as u32)
    } else {
        return Some('\u{FFFD}');
    };
    let mut cp = seed;
    for _ in 0..n_cont {
        let b = read_byte().unwrap_or(0x80);
        if b & 0xC0 != 0x80 {
            return Some('\u{FFFD}');
        }
        cp = (cp << 6) | (b & 0x3F) as u32;
    }
    Some(char::from_u32(cp).unwrap_or('\u{FFFD}'))
}

fn read_integer_token() -> Option<i64> {
    let first = read_non_whitespace_byte()?;
    let mut token = String::new();
    token.push(first as char);

    loop {
        let Some(byte) = read_byte() else {
            break;
        };
        let ch = byte as char;
        if ch.is_whitespace() {
            break;
        }
        token.push(ch);
    }

    token.parse::<i64>().ok()
}

#[unsafe(export_name = "Console.WriteInt")]
pub extern "C" fn console_write_int(value: i64) {
    write_text(&value.to_string());
}

#[unsafe(export_name = "Console.WriteChar")]
pub extern "C" fn console_write_char(value: u32) {
    let ch = char::from_u32(value).unwrap_or('\u{FFFD}');
    let mut buf = [0u8; 4];
    write_text(ch.encode_utf8(&mut buf));
}

#[unsafe(export_name = "Console.WriteLn")]
pub extern "C" fn console_write_ln() {
    write_text("\n");
}

#[unsafe(export_name = "Console.ReadInt")]
pub extern "C" fn console_read_int(out: *mut i64) {
    if out.is_null() {
        return;
    }

    let value = read_integer_token().unwrap_or(0);
    unsafe {
        *out = value;
    }
}

#[unsafe(export_name = "Console.WriteString")]
/// Write a null-terminated `ARRAY OF CHAR` (UTF-32) to the console as UTF-8.
///
/// `ptr` points at the first `u32` element of the array.
/// Writing stops at the first zero code point (null terminator).
pub extern "C" fn console_write_string(ptr: *const u32) {
    if ptr.is_null() {
        return;
    }
    let mut buf = [0u8; 4];
    let mut out = String::new();
    let mut i = 0isize;
    loop {
        let cp = unsafe { *ptr.offset(i) };
        if cp == 0 {
            break;
        }
        let ch = char::from_u32(cp).unwrap_or('\u{FFFD}');
        out.push_str(ch.encode_utf8(&mut buf));
        i += 1;
    }
    write_text(&out);
}

#[unsafe(export_name = "Console.ReadString")]
/// Read a whitespace-delimited token from the console into a `ARRAY OF CHAR` (UTF-32) buffer.
///
/// `out` must point at a caller-allocated buffer of at least `max_len` `u32` elements.
/// `max_len` is the total capacity including the null terminator.
/// The token is decoded from the console's UTF-8 byte stream into UTF-32 code points.
/// The result is always null-terminated; at most `max_len - 1` code points are written.
pub extern "C" fn console_read_string(out: *mut u32, max_len: i64) {
    if out.is_null() || max_len <= 0 {
        return;
    }
    let capacity = (max_len - 1).max(0) as usize;
    let mut count = 0usize;
    // Skip leading whitespace.
    let first = loop {
        match read_utf8_codepoint() {
            None => {
                // EOF before any token: write empty string.
                unsafe { *out = 0 };
                return;
            }
            Some(ch) if ch.is_whitespace() => continue,
            Some(ch) => break ch,
        }
    };
    // Store code points until whitespace, EOF, or buffer full.
    let mut next = Some(first);
    while let Some(ch) = next {
        if ch.is_whitespace() {
            break;
        }
        if count < capacity {
            unsafe { *out.add(count) = ch as u32 };
            count += 1;
        }
        next = read_utf8_codepoint();
    }
    // Null-terminate.
    unsafe { *out.add(count) = 0 };
}

#[unsafe(export_name = "Console.WriteShortString")]
/// Write a null-terminated `ARRAY OF SHORTCHAR` (byte array) to the console.
/// `ptr` points at the first `u8` element; writing stops at the first zero byte.
pub extern "C" fn console_write_short_string(ptr: *const u8) {
    if ptr.is_null() {
        return;
    }
    let s = unsafe { std::ffi::CStr::from_ptr(ptr as *const _) }.to_string_lossy();
    write_text(&s);
}

#[unsafe(export_name = "Console.ReadChar")]
pub extern "C" fn console_read_char(out: *mut u32) {
    if out.is_null() {
        return;
    }
    // Decode a full UTF-8 scalar value from the byte stream so that
    // multi-byte characters (e.g. emoji, non-ASCII letters) arrive as a
    // single CHAR code point rather than a stray leading byte.
    let value = read_utf8_codepoint().map(|c| c as u32).unwrap_or(0);
    unsafe {
        *out = value;
    }
}

pub fn native_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "Console",
            vec![],
            ExportDirectory::new(vec![
                crate::ExportEntry::procedure("WriteInt"),
                crate::ExportEntry::procedure("WriteChar"),
                crate::ExportEntry::procedure("WriteString"),
                crate::ExportEntry::procedure("WriteShortString"),
                crate::ExportEntry::procedure("WriteLn"),
                crate::ExportEntry::procedure("ReadInt"),
                crate::ExportEntry::procedure("ReadChar"),
                crate::ExportEntry::procedure("ReadString"),
            ]),
            "Console.bootstrap",
            "Rust-hosted console I/O facade for tests and JIT execution",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("WriteInt", console_write_int as *const () as usize),
            NativeExportBinding::procedure("WriteChar", console_write_char as *const () as usize),
            NativeExportBinding::procedure("WriteString", console_write_string as *const () as usize),
            NativeExportBinding::procedure("WriteShortString", console_write_short_string as *const () as usize),
            NativeExportBinding::procedure("WriteLn", console_write_ln as *const () as usize),
            NativeExportBinding::procedure("ReadInt", console_read_int as *const () as usize),
            NativeExportBinding::procedure("ReadChar", console_read_char as *const () as usize),
            NativeExportBinding::procedure("ReadString", console_read_string as *const () as usize),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // All console tests share CONSOLE_STATE global, so they must not run in
    // parallel.  Acquire this lock at the top of every test in this module.
    static CONSOLE_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn captured_console_writes_and_reads_integer() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        begin_capture();
        set_input("41 90");

        console_write_int(12);
        console_write_ln();

        let mut value = -1i64;
        console_read_int(&mut value as *mut i64);
        assert_eq!(value, 41);
        assert_eq!(end_capture(), "12\n");
        reset();
    }

    #[test]
    fn captured_console_reads_and_writes_char() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        begin_capture();
        set_input("Az");

        console_write_char('Q' as u32);

        let mut value = 0u32;
        console_read_char(&mut value as *mut u32);
        assert_eq!(value, 'A' as u32);
        assert_eq!(end_capture(), "Q");
        reset();
    }

    #[test]
    fn write_string_converts_utf32_to_utf8() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        begin_capture();

        // "Hi\u{1F600}" (Hi + grinning face emoji) as a null-terminated u32 array.
        let s: Vec<u32> = vec!['H' as u32, 'i' as u32, '\u{1F600}' as u32, 0];
        console_write_string(s.as_ptr());

        assert_eq!(end_capture(), "Hi\u{1F600}");
        reset();
    }

    #[test]
    fn read_string_decodes_utf8_token_into_utf32() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        // UTF-8 input: ASCII word then space then another word.
        set_input("hello world");

        let mut buf = [0u32; 16];
        console_read_string(buf.as_mut_ptr(), 16);
        let result: String = buf.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c).unwrap())
            .collect();
        assert_eq!(result, "hello");
        reset();
    }

    #[test]
    fn read_string_handles_multibyte_utf8_input() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        // UTF-8 encoding of U+00E9 (é): 0xC3 0xA9, followed by a space then more text.
        set_input_bytes(b"caf\xC3\xA9 next");

        let mut buf = [0u32; 16];
        console_read_string(buf.as_mut_ptr(), 16);
        let result: String = buf.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c).unwrap())
            .collect();
        assert_eq!(result, "caf\u{00E9}");
        reset();
    }

    #[test]
    fn read_string_truncates_to_max_len() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        set_input("abcdefgh");

        let mut buf = [0u32; 5]; // capacity 4 chars + null
        console_read_string(buf.as_mut_ptr(), 5);
        let result: String = buf.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c).unwrap())
            .collect();
        assert_eq!(result, "abcd");
        assert_eq!(buf[4], 0); // null terminator always written
        reset();
    }

    #[test]
    fn read_char_decodes_multibyte_utf8() {
        let _guard = CONSOLE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        reset();
        // UTF-8 for U+00E9 (é): 0xC3 0xA9
        set_input_bytes(b"\xC3\xA9");

        let mut value = 0u32;
        console_read_char(&mut value as *mut u32);
        assert_eq!(value, 0x00E9u32);
        reset();
    }
}