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

use std::time::{SystemTime, UNIX_EPOCH};

use crate::gc::{self, BlockHeader, TypeDesc};
use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

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
