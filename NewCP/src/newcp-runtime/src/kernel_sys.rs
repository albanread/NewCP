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
//! - `ThisMod` / `ThisType` / `GetTypeName` / `GetModName` — need the
//!   type-name-on-`TypeDesc` codegen change (also wanted by
//!   heap_introspection).
//! - `PushTrapCleaner` / `PopTrapCleaner` — new runtime feature.
//! - `LastLoaderResult` — needs a global last-failure slot synced from
//!   the loader.
//! - The event-loop primitive (`Kernel.Loop`, `Kernel.Quit`,
//!   `EventSource` registration) — its own focused commit.
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
    ]
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
