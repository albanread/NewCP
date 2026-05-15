//! `BRK` statement runtime — process-state dump for the debugger-
//! breakpoint statement.
//!
//! When a JIT program executes the CP `BRK` statement, control
//! transfers to `__newcp_brk(routine_name, line)`, which writes a
//! structured snapshot of the program's state to stderr and returns.
//! Execution continues after the BRK — it's a snapshot, not a halt.
//!
//! Layout (cheapest first, in case a later section faults):
//!
//!   1. Banner with the BRK site (routine name + source line).
//!   2. Heap summary from `gc::HEAP_COUNTERS`.
//!   3. CPU register state captured via `RtlCaptureContext`.
//!   4. Stack walk via `RtlVirtualUnwind` over the unwind tables
//!      registered by the JIT.  Frames resolve to JIT routine
//!      names through `JIT_SYMBOLS` (populated by the LLVM crate
//!      after MCJIT finalize).
//!
//! ### Safety contract
//!
//! BRK can fire in any state — including a corrupted heap or a
//! partially-constructed object graph.  The dump must not make
//! things worse:
//!
//!   * **No heap allocation.**  Fixed-size stack buffers only.
//!   * **No `format!` / `println!`.**  Numbers formatted by hand.
//!   * **Direct WriteFile** to `STD_ERROR_HANDLE`.
//!   * **Best-effort stack walk.**  Any failure terminates the walk.
//!   * **Never re-enters BRK.**
//!
//! Modelled on NewBCPL's BRK (see `e/NewBCPL/src/newbcpl-runtime/src/brk.rs`).

use std::sync::RwLock;

// ─── JIT-symbol registry ──────────────────────────────────────────

static JIT_SYMBOLS: RwLock<Vec<(u64, String)>> = RwLock::new(Vec::new());

/// Register a JIT-emitted function for stack-trace resolution.
/// Called from `newcp-llvm` after MCJIT finalize, when every
/// function in the module has a stable code-section address.
/// `start_addr` must be the function's entry point.
pub fn register_jit_symbol(start_addr: u64, name: &str) {
    let mut guard = JIT_SYMBOLS.write().expect("JIT_SYMBOLS poisoned");
    let pos = guard.partition_point(|(s, _)| *s < start_addr);
    guard.insert(pos, (start_addr, name.to_string()));
}

/// Reasonable upper bound on a JIT'd routine's machine-code size.
/// Any RIP that sits more than this far above its nearest
/// registered start address is almost certainly host / OS code,
/// not JIT code, and we report it as unnamed.  1 MB is generous
/// yet still tight enough to keep host addresses (typically
/// `0x7FF…`) far from any JIT-d region (`0x1DE…` etc.).
const MAX_REASONABLE_ROUTINE_SIZE: u64 = 1024 * 1024;

fn lookup_jit_symbol(rip: u64) -> Option<String> {
    let guard = JIT_SYMBOLS.read().ok()?;
    if guard.is_empty() {
        return None;
    }
    let after = guard.partition_point(|(s, _)| *s <= rip);
    if after == 0 {
        return None;
    }
    let (start, name) = &guard[after - 1];
    if rip.saturating_sub(*start) > MAX_REASONABLE_ROUTINE_SIZE {
        return None;
    }
    Some(name.clone())
}

// ─── Stderr writer (no heap, no stdio locks) ─────────────────────

#[cfg(windows)]
const BRK_BUFFER_BYTES: usize = 4096;

#[cfg(windows)]
struct BrkWriter {
    buf: [u8; BRK_BUFFER_BYTES],
    pos: usize,
    handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl BrkWriter {
    fn new() -> Self {
        use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE};
        let handle = unsafe { GetStdHandle(STD_ERROR_HANDLE) }
            .unwrap_or(windows::Win32::Foundation::HANDLE::default());
        Self {
            buf: [0; BRK_BUFFER_BYTES],
            pos: 0,
            handle,
        }
    }

    fn flush(&mut self) {
        use windows::Win32::Storage::FileSystem::WriteFile;
        if self.pos == 0 || self.handle.is_invalid() {
            self.pos = 0;
            return;
        }
        let slice = &self.buf[..self.pos];
        let mut written: u32 = 0;
        let _ = unsafe {
            WriteFile(self.handle, Some(slice), Some(&mut written), None)
        };
        self.pos = 0;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        let mut start = 0;
        while start < bytes.len() {
            let space = BRK_BUFFER_BYTES - self.pos;
            let take = (bytes.len() - start).min(space);
            self.buf[self.pos..self.pos + take]
                .copy_from_slice(&bytes[start..start + take]);
            self.pos += take;
            start += take;
            if self.pos == BRK_BUFFER_BYTES {
                self.flush();
            }
        }
    }

    fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    fn write_hex16(&mut self, n: u64) {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let mut tmp = [0u8; 16];
        for i in 0..16 {
            let shift = (15 - i) * 4;
            tmp[i] = HEX[((n >> shift) & 0xF) as usize];
        }
        self.write_bytes(&tmp);
    }

    fn write_dec_i64(&mut self, n: i64) {
        let mut tmp = [0u8; 24];
        let mut len = 0;
        let neg = n < 0;
        let mut v: u64 = if neg {
            (n as i128).unsigned_abs() as u64
        } else {
            n as u64
        };
        if v == 0 {
            tmp[len] = b'0';
            len += 1;
        } else {
            while v > 0 {
                tmp[len] = b'0' + (v % 10) as u8;
                len += 1;
                v /= 10;
            }
        }
        if neg {
            tmp[len] = b'-';
            len += 1;
        }
        tmp[..len].reverse();
        self.write_bytes(&tmp[..len]);
    }

    fn write_dec_u64(&mut self, n: u64) {
        let mut tmp = [0u8; 24];
        let mut len = 0;
        let mut v = n;
        if v == 0 {
            tmp[len] = b'0';
            len += 1;
        } else {
            while v > 0 {
                tmp[len] = b'0' + (v % 10) as u8;
                len += 1;
                v /= 10;
            }
        }
        tmp[..len].reverse();
        self.write_bytes(&tmp[..len]);
    }

    /// Read a null-terminated string from `p` and write it.  Caps
    /// at 256 bytes so a garbage pointer doesn't run forever.
    fn write_cstr(&mut self, p: *const u8) {
        if p.is_null() {
            self.write_str("<null>");
            return;
        }
        unsafe {
            let mut n = 0;
            while n < 256 {
                let b = *p.add(n);
                if b == 0 {
                    break;
                }
                n += 1;
            }
            let slice = core::slice::from_raw_parts(p, n);
            self.write_bytes(slice);
        }
    }
}

// ─── Public entry point ──────────────────────────────────────────

/// Public BRK entry point.  IR-lowering of the CP `BRK` statement
/// emits a call to this with the procedure's mangled name and the
/// source line of the BRK statement.  Both arguments are best-
/// effort; the handler tolerates null / 0.
///
/// `extern "C-unwind"` matches the rest of the runtime ABI (the
/// JIT enables uwtable=2; everything callable from JIT-d code has
/// to participate in unwinding).
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn __newcp_brk(
    routine_name: *const u8,
    line: i64,
) {
    #[cfg(windows)]
    {
        unsafe { brk_impl_windows(routine_name, line) };
    }
    #[cfg(not(windows))]
    {
        unsafe { brk_impl_fallback(routine_name, line) };
    }
}

// ─── Windows implementation ──────────────────────────────────────

#[cfg(windows)]
unsafe fn brk_impl_windows(routine_name: *const u8, line: i64) {
    let mut w = BrkWriter::new();

    // Section 1 — banner.
    w.write_str("\n=== BRK in `");
    w.write_cstr(routine_name);
    if line > 0 {
        w.write_str("` at line ");
        w.write_dec_i64(line);
    } else {
        w.write_str("`");
    }
    w.write_str(" ===\n");
    w.flush();

    // Section 2 — heap summary.
    write_heap_section(&mut w);
    w.flush();

    // Section 3 — register state.
    unsafe { write_context_section(&mut w) };
    w.flush();

    // Section 4 — stack walk.
    unsafe { write_stack_walk_section(&mut w) };
    w.flush();

    w.write_str("=== END BRK ===\n\n");
    w.flush();
}

#[cfg(windows)]
fn write_heap_section(w: &mut BrkWriter) {
    use crate::gc::HEAP_COUNTERS;
    use core::sync::atomic::Ordering;
    let bytes = HEAP_COUNTERS.live_bytes.load(Ordering::Relaxed);
    let blocks = HEAP_COUNTERS.live_blocks.load(Ordering::Relaxed);
    let peak = HEAP_COUNTERS.peak_live_bytes.load(Ordering::Relaxed);
    w.write_str("heap:    live=");
    w.write_dec_u64(bytes);
    w.write_str(" bytes  blocks=");
    w.write_dec_u64(blocks);
    w.write_str("  peak=");
    w.write_dec_u64(peak);
    w.write_str(" bytes\n");
}

// CONTEXT for AMD64 needs 16-byte alignment because it embeds XMM
// register storage.  The `windows` crate's CONTEXT is `#[repr(C)]`
// but doesn't force 16-byte alignment — a stack-zeroed value can
// land on 8 and RtlCaptureContext faults.  Wrap it.
#[cfg(windows)]
#[repr(C, align(16))]
struct AlignedContext(windows::Win32::System::Diagnostics::Debug::CONTEXT);

#[cfg(windows)]
unsafe fn write_context_section(w: &mut BrkWriter) {
    use windows::Win32::System::Diagnostics::Debug::RtlCaptureContext;
    let mut aligned = unsafe { core::mem::zeroed::<AlignedContext>() };
    let ctx = &mut aligned.0;
    ctx.ContextFlags = windows::Win32::System::Diagnostics::Debug::CONTEXT_ALL_AMD64;
    unsafe { RtlCaptureContext(ctx) };

    w.write_str("context: rip=");
    w.write_hex16(ctx.Rip);
    w.write_str("  rsp=");
    w.write_hex16(ctx.Rsp);
    w.write_str("  rbp=");
    w.write_hex16(ctx.Rbp);
    w.write_str("\n         rax=");
    w.write_hex16(ctx.Rax);
    w.write_str("  rbx=");
    w.write_hex16(ctx.Rbx);
    w.write_str("  rcx=");
    w.write_hex16(ctx.Rcx);
    w.write_str("\n         rdx=");
    w.write_hex16(ctx.Rdx);
    w.write_str("  rsi=");
    w.write_hex16(ctx.Rsi);
    w.write_str("  rdi=");
    w.write_hex16(ctx.Rdi);
    w.write_str("\n         r8 =");
    w.write_hex16(ctx.R8);
    w.write_str("  r9 =");
    w.write_hex16(ctx.R9);
    w.write_str("  r10=");
    w.write_hex16(ctx.R10);
    w.write_str("\n         r11=");
    w.write_hex16(ctx.R11);
    w.write_str("  r12=");
    w.write_hex16(ctx.R12);
    w.write_str("  r13=");
    w.write_hex16(ctx.R13);
    w.write_str("\n         r14=");
    w.write_hex16(ctx.R14);
    w.write_str("  r15=");
    w.write_hex16(ctx.R15);
    w.write_str("  flags=");
    w.write_hex16(ctx.EFlags as u64);
    w.write_str("\n");
    let _ = aligned;
}

#[cfg(windows)]
unsafe fn write_stack_walk_section(w: &mut BrkWriter) {
    use windows::Win32::System::Diagnostics::Debug::{
        RtlCaptureContext, RtlLookupFunctionEntry, RtlVirtualUnwind, CONTEXT_ALL_AMD64,
        UNWIND_HISTORY_TABLE, UNW_FLAG_NHANDLER,
    };

    const MAX_FRAMES: usize = 32;

    w.write_str("stack:\n");

    let mut aligned = unsafe { core::mem::zeroed::<AlignedContext>() };
    let ctx = &mut aligned.0;
    ctx.ContextFlags = CONTEXT_ALL_AMD64;
    unsafe { RtlCaptureContext(ctx) };

    let mut history = unsafe { core::mem::zeroed::<UNWIND_HISTORY_TABLE>() };

    for frame_index in 0..MAX_FRAMES {
        let rip = ctx.Rip;
        if rip == 0 {
            break;
        }

        w.write_str("  #");
        w.write_dec_u64(frame_index as u64);
        w.write_str("  rip=");
        w.write_hex16(rip);
        if let Some(name) = lookup_jit_symbol(rip) {
            w.write_str("  in ");
            w.write_bytes(name.as_bytes());
        }
        w.write_str("\n");

        let mut image_base: u64 = 0;
        let func_entry = unsafe {
            RtlLookupFunctionEntry(rip, &mut image_base, Some(&mut history))
        };
        if func_entry.is_null() {
            // Leaf function — no unwind data.  Pop saved RIP off
            // RSP manually.
            let saved_rip_ptr = ctx.Rsp as *const u64;
            if saved_rip_ptr.is_null() {
                break;
            }
            let new_rip = unsafe { core::ptr::read_volatile(saved_rip_ptr) };
            if new_rip == 0 || new_rip == ctx.Rip {
                break;
            }
            ctx.Rip = new_rip;
            ctx.Rsp = ctx.Rsp.wrapping_add(8);
            continue;
        }

        let prev_rip = ctx.Rip;
        let prev_rsp = ctx.Rsp;
        let mut handler_data: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut establisher_frame: u64 = 0;
        let _handler = unsafe {
            RtlVirtualUnwind(
                UNW_FLAG_NHANDLER,
                image_base,
                rip,
                func_entry,
                ctx,
                &mut handler_data,
                &mut establisher_frame,
                None,
            )
        };
        if ctx.Rip == prev_rip && ctx.Rsp == prev_rsp {
            break;
        }
    }
}

// ─── Non-Windows fallback ────────────────────────────────────────

#[cfg(not(windows))]
unsafe fn brk_impl_fallback(routine_name: *const u8, line: i64) {
    use std::io::Write;
    let mut stderr = std::io::stderr().lock();
    let _ = write!(&mut stderr, "\n=== BRK in `");
    if !routine_name.is_null() {
        let mut n = 0usize;
        while n < 256 && unsafe { *routine_name.add(n) } != 0 {
            n += 1;
        }
        let slice = unsafe { core::slice::from_raw_parts(routine_name, n) };
        let _ = stderr.write_all(slice);
    } else {
        let _ = stderr.write_all(b"<null>");
    }
    if line > 0 {
        let _ = write!(&mut stderr, "` at line {line}");
    } else {
        let _ = write!(&mut stderr, "`");
    }
    let _ = writeln!(&mut stderr, " ===");
    let _ = writeln!(&mut stderr, "(non-Windows: register / stack-walk omitted)");
    let _ = writeln!(&mut stderr, "=== END BRK ===\n");
}
