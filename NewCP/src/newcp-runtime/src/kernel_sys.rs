//! Native `KernelSys` / `Kernel` modules — the Rust-backed primitives
//! for the BlackBox-equivalent runtime surface declared in
//! `Mod/KernelSys.cp` and `Mod/Kernel.cp`.
//!
//! This is the "first slice" of the Kernel binding (see
//! docs/stores_module_design.md): Time, Beep, type reflection
//! (TypeOf / TypeBase / TypeSize / LevelOf), and `NewObj`. None of these
//! need codegen changes or new runtime features — they sit directly on
//! top of the existing GC and `TypeDesc` layout.
//!
//! Deferred for follow-up commits:
//! - `GetModName` / `ModOf` — need the TypeDesc.module field wiring
//!   (codegen-side ModuleDesc emission, or a name-based fallback).
//! - `LastLoaderResult` — needs a global last-failure slot synced from
//!   the loader.
//!
//! Done since the original first slice:
//! - `Loop`, `Quit`, `Event`, `EventHandler` (commit 03fab50)
//! - `PushTrapCleaner`, `PopTrapCleaner` (commit 02c78dc)
//! - `GetTypeName`, `GetQualifiedTypeName` (commit 8d9c2bd)
//! - `ThisMod`, `ThisType` (this commit)
//!
//! ABI conventions match the existing `host_file_sys` / `host_date_sys`
//! pattern: every export is `extern "C"`, scalar args are passed by
//! value as `i64`, `VAR` parameters are passed as raw pointers, return
//! values are `i64` (or `()` for void).

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::gc::{self, BlockHeader, TypeDesc};
use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

#[cfg(windows)]
use crate::igui::channels::{self as igui_channels, IGuiEvent};

/// Mask to strip the GC mark bit from a `BlockHeader.tag`.
const MARK_BIT: usize = 1;

// ─── Misc primitives ─────────────────────────────────────────────────────

/// Nanoseconds since the Unix epoch as a 64-bit signed value. CP's
/// `LONGINT` is `i64`; saturates at the i64 max if the system clock is
/// improbably far in the future.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let nanos = d.as_nanos();
            if nanos > i64::MAX as u128 {
                i64::MAX
            } else {
                nanos as i64
            }
        })
        .unwrap_or(0)
}

/// System bell — best effort. Never fails.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_beep() {
    #[cfg(windows)]
    {
        // `MessageBeep(0xFFFFFFFF)` plays the standard system beep on
        // Windows. We avoid pulling in winapi for this single call.
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBeep(uType: u32) -> i32;
        }
        unsafe { MessageBeep(0xFFFF_FFFF) };
    }
    #[cfg(not(windows))]
    {
        // BEL on POSIX terminals. `print!` keeps it stdout-correct
        // under cargo's test capture.
        print!("\x07");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
}

// ─── Type reflection ─────────────────────────────────────────────────────

/// Read the `TypeDesc*` from a heap pointer's block header. `obj` is
/// the payload address (the value an unsafe extern returns from
/// `__newcp_new_rec`); the header sits 16 bytes earlier. Returns 0 if
/// `obj == 0`.
///
/// # Safety
/// The caller asserts that `obj` is either zero or the result of a
/// previous `__newcp_new_rec` call; passing a stack address or a
/// half-freed block here is undefined.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_type_of(obj: i64) -> i64 {
    if obj == 0 {
        return 0;
    }
    let header_size = std::mem::size_of::<BlockHeader>();
    let hdr = (obj as usize).wrapping_sub(header_size) as *const BlockHeader;
    let raw_tag = unsafe { (*hdr).tag };
    (raw_tag & !MARK_BIT) as i64
}

/// Direct base of `t`, or 0 if `t` is a root type. `t` is the
/// `TypeDesc` address as `i64`.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_type_base(t: i64) -> i64 {
    if t == 0 {
        return 0;
    }
    let td = t as *const TypeDesc;
    let base = unsafe { (*td).base };
    base as i64
}

/// Payload size of `t` in bytes (excludes the GC `BlockHeader`).
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_type_size(t: i64) -> i64 {
    if t == 0 {
        return 0;
    }
    let td = t as *const TypeDesc;
    unsafe { (*td).size as i64 }
}

/// Inheritance depth: 0 for a root type, 1 for a direct extension, …
/// Walks the `base` chain. Bounded at 128 hops as a paranoia check
/// against a corrupt cycle.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_level_of(t: i64) -> i64 {
    if t == 0 {
        return 0;
    }
    let mut current = t as *const TypeDesc;
    let mut depth: i64 = 0;
    for _ in 0..128 {
        let base = unsafe { (*current).base };
        if base.is_null() {
            return depth;
        }
        current = base;
        depth += 1;
    }
    depth
}

// ─── Allocation ──────────────────────────────────────────────────────────

/// `NewObj(VAR p: ANYPTR; t: INTEGER)`. Allocates a heap record of
/// runtime type `t` via `__newcp_new_rec` and writes the new payload
/// pointer through `p`.
///
/// # Safety
/// `p` must point to writable memory holding an `ANYPTR`-sized slot.
/// `t` must be 0 (which traps) or the address of a valid live
/// `TypeDesc` whose `size` field is non-negative.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_new_obj(p: *mut *mut u8, t: i64) {
    if p.is_null() {
        panic!("Kernel.NewObj called with NIL VAR pointer");
    }
    if t == 0 {
        panic!("Kernel.NewObj called with NIL type");
    }
    let td = t as *const TypeDesc;
    let payload = unsafe { gc::__newcp_new_rec(td) };
    unsafe { *p = payload };
}

// ─── Event loop ──────────────────────────────────────────────────────────

/// Wire layout of the CP `Kernel.Event` record. Seven `i64`s in
/// declaration order — same shape as the `iGui.NextEvent` VAR-output
/// list, so a Loop handler can pass an Event straight through to
/// lower-level dispatch without re-marshaling.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct CpEvent {
    kind: i64,
    child_id: i64,
    time_ms: i64,
    p1: i64,
    p2: i64,
    p3: i64,
    p4: i64,
}

// Event-kind constants — must stay in sync with the EvX values in
// Mod/Kernel.cp and Mod/iGui.cp.
const EV_NONE: i64 = 0;
const EV_KEY: i64 = 1;
const EV_CHAR: i64 = 2;
const EV_MOUSE: i64 = 3;
const EV_FOCUS: i64 = 4;
const EV_RESIZE: i64 = 5;
const EV_CLOSE: i64 = 7;
const EV_FRAME_CLOSE: i64 = 8;
const EV_MENU: i64 = 9;
const EV_THEME_CHANGE: i64 = 10;
const EV_DPI_CHANGE: i64 = 11;
const EV_TICK: i64 = 13;

/// CP procedure-pointer ABI for `Kernel.EventHandler`. CP `VAR`
/// parameters become raw pointers in the C calling convention; the
/// handler reads/writes `*ev` and `*quit` in place.
type CpEventHandler = extern "C" fn(*mut CpEvent, *mut i64);

/// Cross-thread quit signal. Set by `Kernel.Quit`, polled by
/// `Kernel.Loop` once per iteration. `Relaxed` ordering is sufficient
/// because the loop also re-reads it after every handler call, and
/// the worst-case latency is one event-handling round trip.
static QUIT_SIGNAL: AtomicI64 = AtomicI64::new(0);

/// `Kernel.Quit(code)`. Posts an exit signal observable by the next
/// `Kernel.Loop` iteration. Callable from any thread.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_quit(code: i64) {
    // Reserve 0 for "running" so a single race-free check works.
    let stored = if code == 0 { 1 } else { code };
    QUIT_SIGNAL.store(stored, Ordering::Relaxed);
}

/// Re-entrancy guard. Thread-local so legitimate parallel callers
/// (e.g. cargo test running two `Kernel.Loop` integration tests
/// concurrently) don't trip it; the actual concern this catches is
/// "a handler running inside Loop calls Loop again", which is
/// per-thread by definition.
thread_local! {
    static LOOP_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(windows)]
fn pack_igui_event(ev: IGuiEvent, out: &mut CpEvent) {
    *out = CpEvent::default();
    match ev {
        IGuiEvent::Key {
            child_id,
            vkey,
            scancode,
            mods,
            repeat,
            down,
            time_ms,
        } => {
            out.kind = EV_KEY;
            out.child_id = child_id;
            out.time_ms = time_ms;
            out.p1 = vkey;
            out.p2 = scancode;
            out.p3 = mods;
            out.p4 = (if down { 1 } else { 0 }) | (repeat << 16);
        }
        IGuiEvent::Char {
            child_id,
            codepoint,
            mods,
            time_ms,
        } => {
            out.kind = EV_CHAR;
            out.child_id = child_id;
            out.time_ms = time_ms;
            out.p1 = codepoint;
            out.p2 = mods;
        }
        IGuiEvent::Mouse {
            child_id,
            x,
            y,
            op,
            button,
            mods,
            wheel_delta,
            wheel_lines,
            time_ms,
        } => {
            out.kind = EV_MOUSE;
            out.child_id = child_id;
            out.time_ms = time_ms;
            out.p1 = x;
            out.p2 = y;
            out.p3 = mods | (button << 8) | (op << 16);
            out.p4 = (wheel_delta & 0xFFFF) | (wheel_lines << 16);
        }
        IGuiEvent::Focus { child_id, gained } => {
            out.kind = EV_FOCUS;
            out.child_id = child_id;
            out.p1 = if gained { 1 } else { 0 };
        }
        IGuiEvent::Resize {
            child_id,
            width,
            height,
        } => {
            out.kind = EV_RESIZE;
            out.child_id = child_id;
            out.p1 = width;
            out.p2 = height;
        }
        IGuiEvent::Close { child_id } => {
            out.kind = EV_CLOSE;
            out.child_id = child_id;
        }
        IGuiEvent::FrameClose => {
            out.kind = EV_FRAME_CLOSE;
        }
        IGuiEvent::ThemeChange => {
            out.kind = EV_THEME_CHANGE;
        }
        IGuiEvent::DpiChange {
            child_id,
            dpi_x,
            dpi_y,
        } => {
            out.kind = EV_DPI_CHANGE;
            out.child_id = child_id;
            out.p1 = dpi_x;
            out.p2 = dpi_y;
        }
        IGuiEvent::Menu { menu_id, item_id } => {
            out.kind = EV_MENU;
            out.p1 = menu_id;
            out.p2 = item_id;
        }
        IGuiEvent::Tick { child_id, time_ms } => {
            out.kind = EV_TICK;
            out.child_id = child_id;
            out.time_ms = time_ms;
        }
    }
}

/// Internal idle hook. Runs when no event arrived in the poll window.
/// Today this is a no-op modulo `gc::collect()` pressure heuristics
/// which fire from `__newcp_new_rec` already; future work hangs the
/// loader's `drive_quiescent_collection`, finalizer drain, and a
/// pluggable framework idle list off this function.
fn run_idle_hooks() {
    // Intentionally empty for now. See doc comment.
}

/// `Kernel.Loop(handler)`. The platform-agnostic CP-side event loop.
///
/// Drains the iGui mailbox one event at a time, packs each into a
/// `CpEvent`, and calls the handler. On poll timeout (no event for
/// `IDLE_TIMEOUT_MS`), runs `run_idle_hooks` without invoking the
/// handler. Exits when:
/// - the handler sets `*quit != 0`, or
/// - an `EvFrameClose` event arrives (handler is called once for it,
///   then the loop returns), or
/// - another thread calls `Kernel.Quit`.
///
/// On non-Windows builds the iGui channel is unavailable; the loop
/// becomes a quit-poll spin until `Kernel.Quit` fires. Headless
/// CP programs that don't touch GUI state still get a working
/// event-loop shape this way.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_loop(handler: CpEventHandler) {
    const IDLE_TIMEOUT_MS: i64 = 50;
    let nested = LOOP_DEPTH.with(|d| {
        let prev = d.get();
        d.set(prev + 1);
        prev > 0
    });
    if nested {
        LOOP_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        panic!("Kernel.Loop is not re-entrant on a single thread");
    }
    // Note: do NOT reset QUIT_SIGNAL on entry. A Quit posted *before*
    // Loop is a legitimate pre-arm pattern (used by tests, and by any
    // production code that wants a fail-safe early-exit hook); resetting
    // here would silently swallow it. Quit's contract is "the flag
    // persists until the loop observes it" — and "observe" means
    // "exit". If we ever want to restart Loop after a Quit, that
    // becomes an explicit `Kernel.ResetQuit` call, not an implicit
    // reset on entry.

    let mut quit: i64 = 0;
    while quit == 0 && QUIT_SIGNAL.load(Ordering::Relaxed) == 0 {
        let mut ev = CpEvent::default();
        let got_event;
        #[cfg(windows)]
        {
            got_event = match igui_channels::next_event(IDLE_TIMEOUT_MS) {
                Some(ig) => {
                    pack_igui_event(ig, &mut ev);
                    true
                }
                None => false,
            };
        }
        #[cfg(not(windows))]
        {
            // Headless: just sleep so the spin doesn't peg a CPU.
            std::thread::sleep(std::time::Duration::from_millis(IDLE_TIMEOUT_MS as u64));
            got_event = false;
        }

        if got_event {
            let was_frame_close = ev.kind == EV_FRAME_CLOSE;
            handler(&mut ev as *mut _, &mut quit as *mut _);
            if was_frame_close {
                quit = 1;
            }
        } else {
            run_idle_hooks();
        }
    }

    LOOP_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
}

// ─── Module / Type lookup (Kernel.ThisMod / Kernel.ThisType) ─────────────

/// Process-wide registry of "known" module names — the universe over
/// which `Kernel.ThisMod` succeeds. Populated by:
///   1. The bootstrap shell, which calls
///      [`register_known_module`] for every native-module artifact it
///      registers.
///   2. (Future) the loader, when a compiled CP source module finishes
///      materializing.
///
/// `ThisMod` returns the 1-based index of the name in this Vec
/// (treated as an opaque `Module` handle by CP code). `ThisType`
/// reverse-maps the handle back to the name and walks the heap for
/// the matching TypeDesc.
static MODULE_REGISTRY: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Register `name` as a known module. Idempotent — duplicate names
/// are silently dropped. Public so the runtime's bootstrap and
/// (eventually) the loader can populate the registry.
pub fn register_known_module(name: &str) {
    let mut reg = MODULE_REGISTRY.lock().expect("module registry mutex poisoned");
    if !reg.iter().any(|n| n == name) {
        reg.push(name.to_string());
    }
}

/// Test-only helper: clear the registry. Used by unit tests that
/// need a deterministic starting state across runs.
#[cfg(test)]
fn reset_module_registry_for_test() {
    let mut reg = MODULE_REGISTRY.lock().expect("module registry mutex poisoned");
    reg.clear();
}

/// Decode a UTF-32 zero-terminated codepoint stream into a Rust
/// `String`. CP source identifiers are guaranteed ASCII so the
/// codepoint→char conversion is lossless.
fn read_cp_utf32_string(ptr: *const u32, max_len: i64) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let cap = if max_len <= 0 { 4096 } else { max_len as usize };
    let mut s = String::with_capacity(16);
    for i in 0..cap {
        let cp = unsafe { *ptr.add(i) };
        if cp == 0 {
            break;
        }
        if let Some(c) = char::from_u32(cp) {
            s.push(c);
        }
    }
    s
}

/// `Kernel.ThisMod(IN name: ARRAY OF CHAR): Module`. Looks the name
/// up in the module registry and returns a 1-based handle, or 0
/// (NIL) if no such module is registered. The handle is opaque to
/// CP code — the only thing it does with it is pass it to
/// `Kernel.ThisType`.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_this_mod(name_ptr: *const u32, name_len: i64) -> i64 {
    let name = read_cp_utf32_string(name_ptr, name_len);
    if name.is_empty() {
        return 0;
    }
    let reg = MODULE_REGISTRY.lock().expect("module registry mutex poisoned");
    reg.iter()
        .position(|n| n == &name)
        .map(|idx| (idx + 1) as i64)
        .unwrap_or(0)
}

/// Reverse-map a Module handle (1-based registry index) back to its
/// name. `None` if the handle is 0 or out of range.
fn module_name_from_handle(handle: i64) -> Option<String> {
    if handle <= 0 {
        return None;
    }
    let reg = MODULE_REGISTRY.lock().expect("module registry mutex poisoned");
    reg.get((handle - 1) as usize).cloned()
}

/// Process-wide registry of known TypeDescs, keyed by qualified
/// name. Populated by:
///   1. Codegen-emitted `__newcp_register_type(td)` calls at
///      module-init time (see `__newcp_register_type` below).
///      This is the load-bearing path — every module's
///      `<Module>.__init_types` function calls register_type for
///      each emitted TypeDesc, which the loader runs before
///      `<Module>.body`.
///   2. The test-only [`register_known_type_for_test`] helper for
///      unit tests with hand-fabricated TypeDescs.
///
/// We deliberately do NOT walk the heap to discover TypeDescs:
/// the heap can outlive the JIT image that emitted a TypeDesc
/// (cross-test isolation, future hot reload), so a tag in a
/// surviving block may point at unmapped memory. Registry-based
/// lookup is the only safe path.
static TYPE_DESC_REGISTRY: std::sync::Mutex<Vec<(String, i64)>> =
    std::sync::Mutex::new(Vec::new());

/// Test-only: register a TypeDesc by its qualified name so
/// `kernel_sys_this_type` can find it. Production code populates
/// the registry from codegen (when the per-module-init hook
/// lands); tests use this helper directly.
#[cfg(test)]
fn register_known_type_for_test(qualified_name: &str, td_addr: i64) {
    let mut reg = TYPE_DESC_REGISTRY
        .lock()
        .expect("type registry mutex poisoned");
    reg.retain(|(n, _)| n != qualified_name);
    reg.push((qualified_name.to_string(), td_addr));
}

#[cfg(test)]
fn reset_type_registry_for_test() {
    let mut reg = TYPE_DESC_REGISTRY
        .lock()
        .expect("type registry mutex poisoned");
    reg.clear();
}

/// JIT-callable runtime symbol — codegen emits a call to this
/// function for every TypeDesc the module declares, packed inside
/// a per-module `<Module>.__init_types` function the loader runs
/// before `<Module>.body`.
///
/// Reads the TypeDesc's `name` field (UTF-32 qualified name like
/// `"Stores.StoreDesc"`) and inserts the (name, td) pair into the
/// registry. Idempotent — re-registration with the same name
/// updates the address (e.g. on hot reload).
///
/// # Safety
/// `td` must be a valid `*const TypeDesc` whose `name` field
/// points at a UTF-32 zero-terminated codepoint stream, or null.
#[unsafe(no_mangle)]
pub extern "C" fn __newcp_register_type(td: *const TypeDesc) {
    if td.is_null() {
        return;
    }
    let Some(name) = type_desc_qualified_name_string(td as i64) else {
        return;
    };
    // Mirror the registration into the GC's TypeDesc registry so the
    // loader's RetiredImageDropPredicate can pin a JIT image while
    // any live block still tags one of its TypeDescs. The module name
    // is the qualified name's first segment (e.g. "XMethodChild" in
    // "XMethodChild.ChildDesc").
    let owner_module = name.split('.').next().map(|s| s.to_string());
    crate::gc::register_typedesc(td as usize, owner_module);
    let mut reg = TYPE_DESC_REGISTRY
        .lock()
        .expect("type registry mutex poisoned");
    // Replace existing entry so a re-loaded module's TypeDesc
    // address shadows the prior one.
    reg.retain(|(n, _)| n != &name);
    reg.push((name, td as i64));
}

/// `Kernel.ThisType(m: Module; IN typeName: ARRAY OF CHAR): Type`.
/// Find the TypeDesc whose qualified name is `<module-name>.<typeName>`.
/// Returns the TypeDesc address (treated as `Type` by CP code), or
/// 0 if no matching TypeDesc has been registered.
///
/// Today the registry is empty until codegen-emitted module-init
/// hooks land. CP-side production usage will get NIL back for
/// every call until that work ships; integration tests that need
/// the lookup to succeed populate the registry through the
/// test-only helper.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_this_type(
    m: i64,
    type_name_ptr: *const u32,
    type_name_len: i64,
) -> i64 {
    let Some(module_name) = module_name_from_handle(m) else {
        return 0;
    };
    let type_name = read_cp_utf32_string(type_name_ptr, type_name_len);
    if type_name.is_empty() {
        return 0;
    }
    let qualified = format!("{module_name}.{type_name}");
    let reg = TYPE_DESC_REGISTRY
        .lock()
        .expect("type registry mutex poisoned");
    reg.iter()
        .find(|(n, _)| n == &qualified)
        .map(|(_, addr)| *addr)
        .unwrap_or(0)
}

/// Look up a TypeDesc by its already-qualified name (e.g.
/// "TextModels.StdModelDesc"), encoded as a NUL-terminated 8-bit
/// C string.  Returns the TypeDesc address (suitable for passing
/// to `__newcp_new_rec`) or 0 when no entry matches.
///
/// Used by codegen: when a `NEW(p)` for a cross-module pointer
/// type compiles in a module that doesn't carry the target's
/// TypeDesc global, we fall through to this runtime registry
/// lookup. The qualified name is emitted as a private string
/// constant alongside the call site, so the codegen avoids the
/// extern-global linkage gymnastics that would otherwise be
/// required to share a typed-desc pointer across compiled CP
/// modules.
#[unsafe(no_mangle)]
pub extern "C" fn __newcp_lookup_typedesc(qualified_name: *const u8) -> i64 {
    if qualified_name.is_null() {
        return 0;
    }
    // Read the 0-terminated UTF-8 / Latin-1 byte sequence the
    // codegen emits.
    let mut len = 0usize;
    while unsafe { *qualified_name.add(len) } != 0 {
        len += 1;
        if len > 4096 {
            return 0;
        }
    }
    let bytes = unsafe { std::slice::from_raw_parts(qualified_name, len) };
    let name = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let reg = TYPE_DESC_REGISTRY
        .lock()
        .expect("type registry mutex poisoned");
    reg.iter()
        .find(|(n, _)| n == name)
        .map(|(_, addr)| *addr)
        .unwrap_or(0)
}

// ─── Type-name lookup ────────────────────────────────────────────────────

/// Read the UTF-32 zero-terminated name attached to a `TypeDesc` and
/// return its character count (excluding the terminator). Returns
/// `None` if the TypeDesc has no name, returns the address of the
/// codepoint stream and the number of codepoints if it does.
#[inline]
fn type_desc_name_codepoints(t: i64) -> Option<(*const u32, usize)> {
    if t == 0 {
        return None;
    }
    let td = t as *const TypeDesc;
    let name_ptr = unsafe { (*td).name };
    if name_ptr.is_null() {
        return None;
    }
    let mut len = 0usize;
    unsafe {
        while *name_ptr.add(len) != 0 {
            len += 1;
            if len > 4096 {
                // Defensive: a runaway scan probably means a corrupt
                // TypeDesc. Don't loop forever.
                return Some((name_ptr, len));
            }
        }
    }
    Some((name_ptr, len))
}

/// Helper: copy `len` codepoints from `src` into the OUT array
/// `dst` (capped at `cap - 1`), then write a zero terminator.
/// Mirrors the CP `OUT name: ARRAY OF CHAR` ABI: `dst` is a pointer
/// to a UTF-32 codepoint buffer and `cap` is the hidden length
/// argument the open-array convention appends.
#[inline]
fn write_codepoints_to_out_array(src: *const u32, len: usize, dst: *mut u32, cap: i64) {
    if dst.is_null() || cap <= 0 {
        return;
    }
    let cap_chars = (cap as usize).saturating_sub(1); // reserve room for terminator
    let n = len.min(cap_chars);
    unsafe {
        for i in 0..n {
            *dst.add(i) = *src.add(i);
        }
        *dst.add(n) = 0;
    }
}

/// `Kernel.GetTypeName(t: Type; OUT name: ARRAY OF CHAR)`. Returns
/// the *bare* type name (the suffix after the last `.`) — matches
/// the legacy BlackBox semantics where `t.mod.name + "." +
/// GetTypeName(t)` produced the qualified form.
///
/// `name` is the OUT array's payload pointer; `name_len` is the
/// hidden length argument the CP open-array ABI appends.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_get_type_name(t: i64, name: *mut u32, name_len: i64) {
    let Some((src, len)) = type_desc_name_codepoints(t) else {
        // Empty string when no name available.
        if !name.is_null() && name_len > 0 {
            unsafe { *name = 0 };
        }
        return;
    };
    // Find the last `.` (codepoint 0x2E) and start after it.
    let mut bare_start = 0usize;
    for i in 0..len {
        let cp = unsafe { *src.add(i) };
        if cp == 0x2E {
            bare_start = i + 1;
        }
    }
    let bare_src = unsafe { src.add(bare_start) };
    let bare_len = len - bare_start;
    write_codepoints_to_out_array(bare_src, bare_len, name, name_len);
}

/// `KernelSys.GetQualifiedTypeName(t: Type; OUT name: ARRAY OF CHAR)`.
/// Writes the full qualified name (e.g. `"Stores.StoreDesc"`) into
/// the OUT array. Used by `heap_introspect` and by any code that
/// wants the qualified form without composing it from parts.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_get_qualified_type_name(t: i64, name: *mut u32, name_len: i64) {
    let Some((src, len)) = type_desc_name_codepoints(t) else {
        if !name.is_null() && name_len > 0 {
            unsafe { *name = 0 };
        }
        return;
    };
    write_codepoints_to_out_array(src, len, name, name_len);
}

/// Internal helper for `heap_introspect`: build a Rust `String` from
/// a TypeDesc address. ASCII-only by language rule, so the
/// codepoint→char conversion is lossless.
pub(crate) fn type_desc_qualified_name_string(t: i64) -> Option<String> {
    let (src, len) = type_desc_name_codepoints(t)?;
    let mut s = String::with_capacity(len);
    for i in 0..len {
        let cp = unsafe { *src.add(i) };
        if let Some(c) = char::from_u32(cp) {
            s.push(c);
        }
    }
    Some(s)
}

// ─── Trap-cleaner stack ──────────────────────────────────────────────────

/// LIFO stack of registered trap cleaners (CP `Kernel.TrapCleaner`
/// payload pointers). Thread-local because traps fire on the same
/// thread that triggered them; each language thread gets its own
/// recovery scope.
///
/// Each entry is the *payload* pointer of a TrapCleaner record — the
/// same value `Kernel.PushTrapCleaner` received. The runtime reads
/// the block header at `payload - 16` to get the `TypeDesc`, then
/// dispatches `Cleanup` via vtable slot 0.
thread_local! {
    static TRAP_CLEANERS: std::cell::RefCell<Vec<*mut u8>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Set while inside `run_trap_cleaners`. A cleaner that traps would
/// otherwise re-enter the walker and either loop forever or
/// double-fire registered cleaners; the guard short-circuits the
/// nested call to a hard abort.
thread_local! {
    static IN_TRAP_RECOVERY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `Kernel.PushTrapCleaner(c: TrapCleaner)`. Pushes `cleaner` onto
/// the thread-local recovery stack. The runtime invokes `Cleanup`
/// LIFO if a trap fires before a matching `PopTrapCleaner`.
///
/// `cleaner` must be a non-NIL CP heap pointer — the payload of a
/// `Kernel.TrapCleaner`-derived record allocated through the GC.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_push_trap_cleaner(cleaner: *mut u8) {
    if cleaner.is_null() {
        panic!("Kernel.PushTrapCleaner called with NIL cleaner");
    }
    TRAP_CLEANERS.with(|stack| stack.borrow_mut().push(cleaner));
}

/// `Kernel.PopTrapCleaner(c: TrapCleaner)`. Pops the top cleaner.
/// `cleaner` MUST equal the value most recently pushed; an
/// unbalanced pop traps. The error message names both pointers so
/// the calling-site mismatch is visible in a debug build.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_pop_trap_cleaner(cleaner: *mut u8) {
    TRAP_CLEANERS.with(|stack| {
        let mut stack = stack.borrow_mut();
        let top = stack.pop();
        match top {
            None => panic!("Kernel.PopTrapCleaner: stack empty"),
            Some(t) if t == cleaner => {}
            Some(t) => panic!(
                "Kernel.PopTrapCleaner: stack imbalance — top {:p}, popped {:p}",
                t, cleaner
            ),
        }
    });
}

/// Walk the cleaner stack LIFO, invoking each `Cleanup` method, and
/// drain the stack as we go. Called from `__newcp_trap` just before
/// process abort, and exposed for unit-test scaffolding.
///
/// A cleaner is invoked by reading the block header at `cleaner - 16`,
/// extracting the `TypeDesc`, reading vtable slot 0 (where the CP
/// frontend places `TrapCleanerDesc.Cleanup`), and calling it with
/// the cleaner pointer as the receiver.
///
/// # Safety
/// Each pushed cleaner must remain a live GC allocation until popped.
/// If the GC reclaimed a block whose payload is still on this stack,
/// dispatch reads freed memory.
pub fn run_trap_cleaners() {
    // Re-entrancy guard. If a cleaner itself traps, we don't re-walk
    // the (already-being-drained) stack.
    let already_running = IN_TRAP_RECOVERY.with(|g| {
        let prev = g.get();
        g.set(true);
        prev
    });
    if already_running {
        eprintln!("[trap] cleaner trapped during recovery; aborting without further cleanup");
        return;
    }

    // Drain LIFO. Take the whole stack out so a cleaner that
    // accidentally calls Push during its run doesn't extend our walk.
    let mut stack: Vec<*mut u8> = TRAP_CLEANERS.with(|s| std::mem::take(&mut *s.borrow_mut()));

    while let Some(cleaner) = stack.pop() {
        if cleaner.is_null() {
            continue;
        }
        // SAFETY: cleaner is a payload pointer; its block header sits
        // 16 bytes earlier. The TypeDesc is whatever was tagged when
        // the record was NEW'd, with the GC mark bit potentially
        // set; mask the LSB.
        unsafe {
            let header_size = std::mem::size_of::<BlockHeader>();
            let hdr = (cleaner as usize - header_size) as *const BlockHeader;
            let raw_tag = (*hdr).tag;
            let td = (raw_tag & !MARK_BIT) as *const TypeDesc;
            if td.is_null() || (*td).vtable.is_null() || (*td).vtable_len == 0 {
                eprintln!(
                    "[trap] cleaner at {:p} has no usable vtable; skipping",
                    cleaner
                );
                continue;
            }
            // Slot 0 is `Cleanup` — the only NEW method declared on
            // Kernel.TrapCleanerDesc, so subclass vtables put their
            // override at slot 0.
            let slot_ptr = (*td).vtable;
            let fn_addr = *slot_ptr;
            if fn_addr.is_null() {
                eprintln!(
                    "[trap] cleaner at {:p} has NULL Cleanup slot; skipping",
                    cleaner
                );
                continue;
            }
            type CleanupFn = extern "C" fn(*mut u8);
            let cleanup: CleanupFn = std::mem::transmute(fn_addr);
            cleanup(cleaner);
        }
    }

    IN_TRAP_RECOVERY.with(|g| g.set(false));
}

// ─── Native module registrations ─────────────────────────────────────────

/// The set of (cp_name, fn_ptr) entries shared by both `KernelSys` and
/// `Kernel` registrations. The two CP modules expose the same primitives
/// under different name conventions; one Rust function backs each pair.
fn kernel_exports() -> Vec<(&'static str, *const ())> {
    vec![
        ("Time",     kernel_sys_time      as *const ()),
        ("Beep",     kernel_sys_beep      as *const ()),
        ("TypeOf",   kernel_sys_type_of   as *const ()),
        ("TypeBase", kernel_sys_type_base as *const ()),
        ("BaseOf",   kernel_sys_type_base as *const ()),
        ("TypeSize", kernel_sys_type_size as *const ()),
        ("SizeOf",   kernel_sys_type_size as *const ()),
        ("LevelOf",  kernel_sys_level_of  as *const ()),
        ("NewObj",   kernel_sys_new_obj   as *const ()),
        ("Loop",     kernel_sys_loop      as *const ()),
        ("Quit",     kernel_sys_quit      as *const ()),
        ("PushTrapCleaner", kernel_sys_push_trap_cleaner as *const ()),
        ("PopTrapCleaner",  kernel_sys_pop_trap_cleaner  as *const ()),
        ("GetTypeName",     kernel_sys_get_type_name     as *const ()),
        ("GetQualifiedTypeName", kernel_sys_get_qualified_type_name as *const ()),
        ("ThisMod",         kernel_sys_this_mod          as *const ()),
        ("ThisType",        kernel_sys_this_type         as *const ()),
        ("Collect",         kernel_sys_collect           as *const ()),
    ]
}

/// `Kernel.Collect()` — explicit GC cycle.  Called from CP code that
/// wants to force a stop-the-world mark + sweep + finalizer drain.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_sys_collect() {
    crate::gc::collect();
}

fn build_artifact(module_name: &str, summary: &'static str) -> NativeModuleArtifact {
    let entries = kernel_exports();
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            module_name,
            vec![],
            ExportDirectory::new(
                entries.iter().map(|(n, _)| ExportEntry::procedure(*n)).collect(),
            ),
            format!("{module_name}.bootstrap"),
            summary,
            vec![],
        ),
        entries
            .iter()
            .map(|(n, p)| NativeExportBinding::procedure(*n, *p as usize))
            .collect(),
    )
}

pub fn kernel_sys_native_module_artifact() -> NativeModuleArtifact {
    build_artifact(
        "KernelSys",
        "Rust-hosted flat-API Kernel primitives (Time, type reflection, NewObj)",
    )
}

pub fn kernel_native_module_artifact() -> NativeModuleArtifact {
    build_artifact(
        "Kernel",
        "Rust-hosted typed Kernel surface (Time, type reflection, NewObj)",
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gc::{
        __newcp_init_gc, __newcp_new_rec, lock_tests_global, reset_for_test, Finalizer,
    };

    fn lock_tests() -> std::sync::MutexGuard<'static, ()> {
        lock_tests_global()
    }

    fn make_leaf_typedesc(payload_size: usize) -> *const TypeDesc {
        #[repr(C)]
        struct LeafTd {
            base: TypeDesc,
            sentinel: isize,
        }
        let td = Box::new(LeafTd {
            base: TypeDesc {
                size: payload_size as isize,
                module: std::ptr::null(),
                finalizer: None as Option<Finalizer>,
                base: std::ptr::null(),
                vtable: std::ptr::null(),
                vtable_len: 0,
                name: std::ptr::null(),
                ptroffs: [],
            },
            sentinel: -1,
        });
        Box::into_raw(td) as *const TypeDesc
    }

    fn make_extension_typedesc(payload_size: usize, base: *const TypeDesc) -> *const TypeDesc {
        #[repr(C)]
        struct LeafTd {
            base: TypeDesc,
            sentinel: isize,
        }
        let td = Box::new(LeafTd {
            base: TypeDesc {
                size: payload_size as isize,
                module: std::ptr::null(),
                finalizer: None as Option<Finalizer>,
                base,
                vtable: std::ptr::null(),
                vtable_len: 0,
                name: std::ptr::null(),
                ptroffs: [],
            },
            sentinel: -1,
        });
        Box::into_raw(td) as *const TypeDesc
    }

    #[test]
    fn time_is_monotonic_within_a_pair_of_reads() {
        let t1 = kernel_sys_time();
        let t2 = kernel_sys_time();
        assert!(t2 >= t1, "Time should not go backwards: {t1} → {t2}");
        assert!(t1 > 0, "Time should be positive after epoch");
    }

    #[test]
    fn beep_does_not_panic() {
        // Just verify we can call it without aborting.
        kernel_sys_beep();
    }

    #[test]
    fn type_reflection_round_trips_through_alloc() {
        let _t = lock_tests();
        reset_for_test();

        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let leaf_td = make_leaf_typedesc(64);
        let ext_td = make_extension_typedesc(96, leaf_td);

        // Allocate one of each and verify TypeOf round-trips.
        let leaf = unsafe { __newcp_new_rec(leaf_td) };
        let ext = unsafe { __newcp_new_rec(ext_td) };

        assert_eq!(
            kernel_sys_type_of(leaf as i64),
            leaf_td as i64,
            "TypeOf(leaf) must equal its declared TypeDesc"
        );
        assert_eq!(
            kernel_sys_type_of(ext as i64),
            ext_td as i64,
            "TypeOf(ext) must equal its declared TypeDesc"
        );

        // Sizes match what the descriptors say.
        assert_eq!(kernel_sys_type_size(leaf_td as i64), 64);
        assert_eq!(kernel_sys_type_size(ext_td as i64), 96);

        // Base + level: leaf is root (0), ext is direct child (1).
        assert_eq!(kernel_sys_type_base(leaf_td as i64), 0);
        assert_eq!(kernel_sys_type_base(ext_td as i64), leaf_td as i64);
        assert_eq!(kernel_sys_level_of(leaf_td as i64), 0);
        assert_eq!(kernel_sys_level_of(ext_td as i64), 1);

        // NIL handles short-circuit safely.
        assert_eq!(kernel_sys_type_of(0), 0);
        assert_eq!(kernel_sys_type_base(0), 0);
        assert_eq!(kernel_sys_type_size(0), 0);
        assert_eq!(kernel_sys_level_of(0), 0);
    }

    extern "C" fn quit_immediately_handler(_ev: *mut CpEvent, quit: *mut i64) {
        unsafe { *quit = 1 };
    }

    #[test]
    fn loop_exits_when_quit_signal_set_before_entry() {
        // QUIT_SIGNAL is process-global; serialise with the rest of
        // the runtime test suite so concurrent runs don't race.
        let _t = lock_tests();
        QUIT_SIGNAL.store(0, Ordering::Relaxed);

        // No GUI thread is running in this test process, so the loop
        // would otherwise spin forever waiting for events. Pre-arming
        // the quit signal makes the very first iteration exit.
        kernel_sys_quit(7);
        kernel_sys_loop(quit_immediately_handler);

        // Reset for downstream tests — Quit(0) is *not* a reset
        // (`0` is remapped to `1` to keep the "set" sentinel
        // unambiguous); only a direct store clears the flag.
        QUIT_SIGNAL.store(0, Ordering::Relaxed);
    }

    #[test]
    fn quit_signal_round_trips() {
        let _t = lock_tests();
        QUIT_SIGNAL.store(0, Ordering::Relaxed);

        // Quit always stores a non-zero "stopping" sentinel, even
        // when called with code 0 — that way the loop's poll
        // unambiguously distinguishes "not requested" (== 0) from
        // "requested" (anything else).
        kernel_sys_quit(42);
        assert_eq!(QUIT_SIGNAL.load(Ordering::Relaxed), 42);
        kernel_sys_quit(0);
        assert_ne!(QUIT_SIGNAL.load(Ordering::Relaxed), 0);

        QUIT_SIGNAL.store(0, Ordering::Relaxed);
    }

    #[test]
    fn trap_cleaners_fire_lifo_and_drain_the_stack() {
        let _t = lock_tests();
        reset_for_test();
        TRAP_CLEANERS.with(|s| s.borrow_mut().clear());

        // Side-effect log: each cleaner appends an i64 here so we
        // can verify the LIFO order.
        static CALL_LOG: std::sync::Mutex<Vec<i64>> = std::sync::Mutex::new(Vec::new());
        CALL_LOG.lock().unwrap().clear();

        // Two distinct fake "Cleanup" implementations. Each takes
        // the receiver pointer and reads its first 8 bytes as a
        // tag value so we know which cleaner ran.
        extern "C" fn cleanup_a(self_ptr: *mut u8) {
            let tag = unsafe { *(self_ptr as *const i64) };
            CALL_LOG.lock().unwrap().push(tag);
        }
        extern "C" fn cleanup_b(self_ptr: *mut u8) {
            let tag = unsafe { *(self_ptr as *const i64) };
            CALL_LOG.lock().unwrap().push(tag * 1000);
        }

        // Build two TypeDescs, each with a 1-slot vtable pointing at
        // a different cleanup function. We leak the boxes so the
        // pointers stay live for the rest of the process.
        fn make_typedesc_with_cleanup(cleanup: extern "C" fn(*mut u8)) -> *const TypeDesc {
            #[repr(C)]
            struct TdWithVtable {
                base: TypeDesc,
                sentinel: isize,
            }
            let vtable_box: Box<[*const ()]> = Box::new([cleanup as *const ()]);
            let vtable_ptr = Box::leak(vtable_box).as_ptr() as *const *const ();
            let td = Box::new(TdWithVtable {
                base: TypeDesc {
                    size: 16,
                    module: std::ptr::null(),
                    finalizer: None as Option<Finalizer>,
                    base: std::ptr::null(),
                    vtable: vtable_ptr,
                    vtable_len: 1,
                    name: std::ptr::null(),
                    ptroffs: [],
                },
                sentinel: -1,
            });
            Box::into_raw(td) as *const TypeDesc
        }

        let td_a = make_typedesc_with_cleanup(cleanup_a);
        let td_b = make_typedesc_with_cleanup(cleanup_b);

        // Allocate two cleaner blocks via the GC so they have real
        // BlockHeaders the walker can read.
        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let cleaner_a = unsafe { __newcp_new_rec(td_a) };
        let cleaner_b = unsafe { __newcp_new_rec(td_b) };

        // Tag each payload's first 8 bytes so cleanup_* can read it.
        unsafe {
            *(cleaner_a as *mut i64) = 1;
            *(cleaner_b as *mut i64) = 2;
        }

        // Push A then B; expect Cleanup to fire B (× 1000) before A.
        kernel_sys_push_trap_cleaner(cleaner_a);
        kernel_sys_push_trap_cleaner(cleaner_b);
        run_trap_cleaners();

        let log = CALL_LOG.lock().unwrap().clone();
        assert_eq!(log, vec![2 * 1000, 1], "cleaners must fire LIFO");

        // Stack must be empty afterwards.
        let remaining = TRAP_CLEANERS.with(|s| s.borrow().len());
        assert_eq!(remaining, 0, "run_trap_cleaners must drain the stack");
    }

    #[test]
    fn pop_trap_cleaner_balances_with_push() {
        let _t = lock_tests();
        TRAP_CLEANERS.with(|s| s.borrow_mut().clear());

        // Use raw addresses; the cleaner pointers don't have to
        // resolve through the vtable here because we never call
        // run_trap_cleaners in this test — we only exercise the
        // push/pop balance check.
        let a = 0x1000usize as *mut u8;
        let b = 0x2000usize as *mut u8;

        kernel_sys_push_trap_cleaner(a);
        kernel_sys_push_trap_cleaner(b);
        kernel_sys_pop_trap_cleaner(b);
        kernel_sys_pop_trap_cleaner(a);

        let depth = TRAP_CLEANERS.with(|s| s.borrow().len());
        assert_eq!(depth, 0);
    }

    #[test]
    fn this_mod_returns_handle_for_registered_module() {
        let _t = lock_tests();
        reset_module_registry_for_test();

        register_known_module("Stores");
        register_known_module("TextModels");

        // UTF-32 zero-terminated buffer for "Stores".
        let stores_utf32: Vec<u32> = "Stores".chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let h = kernel_sys_this_mod(stores_utf32.as_ptr(), stores_utf32.len() as i64);
        assert_eq!(h, 1, "first-registered module gets handle 1");

        let textmodels_utf32: Vec<u32> =
            "TextModels".chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let h2 = kernel_sys_this_mod(textmodels_utf32.as_ptr(), textmodels_utf32.len() as i64);
        assert_eq!(h2, 2);

        // Unknown module returns 0.
        let unknown_utf32: Vec<u32> =
            "DoesNotExist".chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let h3 = kernel_sys_this_mod(unknown_utf32.as_ptr(), unknown_utf32.len() as i64);
        assert_eq!(h3, 0);

        // Empty name returns 0.
        let empty: Vec<u32> = vec![0];
        let h4 = kernel_sys_this_mod(empty.as_ptr(), 1);
        assert_eq!(h4, 0);
    }

    #[test]
    fn this_type_resolves_registered_typedesc() {
        let _t = lock_tests();
        reset_module_registry_for_test();
        reset_type_registry_for_test();

        register_known_module("MyMod");
        // Use a fake TypeDesc address (just a non-zero value).
        let fake_td_addr: i64 = 0xCAFEBABE_DEADBEEF_u64 as i64;
        register_known_type_for_test("MyMod.MyDesc", fake_td_addr);

        // ThisMod("MyMod") → handle.
        let mod_utf32: Vec<u32> =
            "MyMod".chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let m = kernel_sys_this_mod(mod_utf32.as_ptr(), mod_utf32.len() as i64);
        assert_ne!(m, 0);

        // ThisType(m, "MyDesc") → fake_td_addr.
        let type_utf32: Vec<u32> =
            "MyDesc".chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let found = kernel_sys_this_type(m, type_utf32.as_ptr(), type_utf32.len() as i64);
        assert_eq!(found, fake_td_addr);

        // Unknown type returns 0.
        let bogus_utf32: Vec<u32> =
            "Bogus".chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let bogus = kernel_sys_this_type(m, bogus_utf32.as_ptr(), bogus_utf32.len() as i64);
        assert_eq!(bogus, 0);

        // Bad module handle returns 0.
        let bad = kernel_sys_this_type(99999, type_utf32.as_ptr(), type_utf32.len() as i64);
        assert_eq!(bad, 0);
    }

    #[test]
    fn get_qualified_type_name_reads_codegen_emitted_name() {
        // Build a TypeDesc by hand that points at a UTF-32 codepoint
        // string; verify the shim reads it back through the same path
        // CP code uses.
        let utf32: Vec<u32> = "Stores.StoreDesc"
            .chars()
            .map(|c| c as u32)
            .chain(std::iter::once(0u32))
            .collect();
        let utf32_box = utf32.into_boxed_slice();
        let utf32_ptr = Box::leak(utf32_box).as_ptr();

        #[repr(C)]
        struct LeafTd {
            base: TypeDesc,
            sentinel: isize,
        }
        let td = Box::new(LeafTd {
            base: TypeDesc {
                size: 16,
                module: std::ptr::null(),
                finalizer: None as Option<Finalizer>,
                base: std::ptr::null(),
                vtable: std::ptr::null(),
                vtable_len: 0,
                name: utf32_ptr,
                ptroffs: [],
            },
            sentinel: -1,
        });
        let td_ptr = Box::into_raw(td) as *const TypeDesc;

        // Read the qualified name via the shim.
        let mut buf = [0u32; 64];
        kernel_sys_get_qualified_type_name(td_ptr as i64, buf.as_mut_ptr(), 64);
        let read_back: String = buf
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c).unwrap())
            .collect();
        assert_eq!(read_back, "Stores.StoreDesc");

        // GetTypeName returns the bare suffix.
        let mut buf2 = [0u32; 64];
        kernel_sys_get_type_name(td_ptr as i64, buf2.as_mut_ptr(), 64);
        let bare: String = buf2
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c).unwrap())
            .collect();
        assert_eq!(bare, "StoreDesc");
    }

    // Note: there is no unit test for the imbalance-traps case.
    // `kernel_sys_pop_trap_cleaner` panics on imbalance, but
    // panicking through an `extern "C"` boundary aborts the
    // process under the default unwind strategy, which would
    // tear down the test runner. The behaviour is documented;
    // production callers detect imbalance through the abort and
    // its diagnostic message.

    #[test]
    fn new_obj_writes_payload_through_var_pointer() {
        let _t = lock_tests();
        reset_for_test();

        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let td = make_leaf_typedesc(32);

        let mut slot: *mut u8 = std::ptr::null_mut();
        kernel_sys_new_obj(&mut slot as *mut *mut u8, td as i64);

        assert!(!slot.is_null(), "NewObj must allocate a non-null payload");
        // The new block must report `td` as its type.
        assert_eq!(kernel_sys_type_of(slot as i64), td as i64);
    }
}
