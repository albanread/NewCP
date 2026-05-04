//! Memory management and Garbage Collection for NewCP.
//!
//! Implements a Mark-and-Sweep GC modelled on the BlackBox Component Pascal
//! runtime, with:
//! - **Conservative stack scanning** — no precise stack maps required from LLVM.
//! - **Precise heap tracing** — every object carries a `TypeDesc` with pointer offsets.
//! - **Precise global tracing** — each module registers its `varBase` struct and
//!   the byte offsets of all pointer fields within it.
//!
//! # Heap layout
//! The managed heap is composed of one or more **Clusters** — large, contiguous
//! OS allocations subdivided into linearly-walkable **Blocks**. Each block
//! begins with a `BlockHeader` and is followed by its payload (or, for free
//! blocks, by a free-list link in the payload region). New objects are
//! allocated either from a per-cluster free list (fed by sweep) or by bumping
//! the cluster's high-water mark. When neither succeeds, `__newcp_new_rec`
//! triggers a `collect()` and retries; if that still fails, it grows the heap
//! by adding a new cluster.

use std::alloc::Layout;
use std::sync::Mutex;
use std::thread::ThreadId;

// ─────────────────────────────────────────────────────────────────────────────
// BlockHeader
// ─────────────────────────────────────────────────────────────────────────────

/// Header prefixed to every block in a cluster (allocated *or* free).
///
/// JIT-compiled code receives a pointer to the *payload* region,
/// `size_of::<BlockHeader>()` bytes past this struct. The GC recovers the
/// header by subtracting that fixed offset.
///
/// 64-bit layout (16 bytes):
/// ```text
///   +0 : tag        — TypeDesc pointer | GC mark bit (LSB)
///                     A `tag` whose value (with mark bit stripped) is 0
///                     denotes a **free** block; the payload then begins with
///                     a `FreeBlockLink` to the next free block in the cluster.
///   +8 : block_size — total block size in bytes (header + payload).
///                     Used to walk the cluster linearly during sweep and
///                     during conservative root resolution.
/// ```
#[repr(C)]
pub struct BlockHeader {
    /// `TypeDesc` address with the GC mark bit packed into the LSB.
    /// Use `type_desc()` to obtain a clean, dereferenceable pointer.
    /// A value of `0` (mark bit cleared) indicates a free block.
    pub tag: usize,
    /// Total block size in bytes, including this header. Required to walk the
    /// cluster linearly during sweep and to split free blocks during alloc.
    pub block_size: usize,
}

impl BlockHeader {
    const MARK_BIT: usize = 1;

    #[inline]
    pub fn is_marked(&self) -> bool {
        self.tag & Self::MARK_BIT != 0
    }

    #[inline]
    pub fn set_mark(&mut self) {
        self.tag |= Self::MARK_BIT;
    }

    #[inline]
    pub fn clear_mark(&mut self) {
        self.tag &= !Self::MARK_BIT;
    }

    /// Returns a typed pointer to this block's `TypeDesc`, with the mark bit stripped.
    #[inline]
    pub fn type_desc(&self) -> *const TypeDesc {
        (self.tag & !Self::MARK_BIT) as *const TypeDesc
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TypeDesc
// ─────────────────────────────────────────────────────────────────────────────

/// Optional finalizer signature. Invoked once on a block right before it is
/// returned to the free list during sweep. Receives a pointer to the block's
/// payload (the same pointer originally returned by `__newcp_new_rec`).
///
/// Finalizers must not allocate, must not retain the pointer past the call,
/// and must not perform GC-visible work. They are intended for native resource
/// release (file handles, OS handles, COM `Release`, etc.).
pub type Finalizer = unsafe extern "C" fn(*mut u8);

/// Runtime type descriptor emitted by `newcp-llvm` for every heap-allocated type.
///
/// `newcp-llvm` synthesises one `TypeDesc` global constant per `RECORD` type.
/// The `ptroffs` trailing array lists payload byte offsets of all pointer-typed
/// fields, terminated by the first **negative** entry (sentinel, e.g. `-1`).
///
/// The `[isize; 0]` trailing field is a DST proxy for the variably-sized
/// compiler-emitted array; callers access elements through raw pointer arithmetic
/// via `pointer_offsets()`.
#[repr(C)]
pub struct TypeDesc {
    /// Payload size in bytes (does **not** include `BlockHeader`).
    pub size: isize,
    /// Owning module, or null for built-in types.
    pub module: *const ModuleDesc,
    /// Optional finalizer; `None` if the type requires no cleanup.
    /// Same ABI as a nullable function pointer.
    pub finalizer: Option<Finalizer>,
    /// Sentinel-terminated (first negative entry) array of payload byte offsets
    /// where heap-pointer fields reside.
    pub ptroffs: [isize; 0],
}

// TypeDesc constants are emitted read-only by the compiler and are safe to
// share across threads.
unsafe impl Sync for TypeDesc {}
unsafe impl Send for TypeDesc {}

impl TypeDesc {
    /// Returns an iterator over the non-negative pointer offsets in `ptroffs`.
    ///
    /// # Safety
    /// The `ptroffs` array must be properly terminated by a negative sentinel
    /// value and must remain valid for the lifetime of the returned iterator.
    pub unsafe fn pointer_offsets(&self) -> impl Iterator<Item = isize> {
        let mut idx = 0usize;
        let base = self.ptroffs.as_ptr();
        std::iter::from_fn(move || unsafe {
            let offset = *base.add(idx);
            if offset < 0 {
                None
            } else {
                idx += 1;
                Some(offset)
            }
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ModuleDesc
// ─────────────────────────────────────────────────────────────────────────────

/// Module-level root metadata emitted by `newcp-llvm` alongside each compiled
/// module's `%ModuleName.Data` global struct.
///
/// `newcp-llvm` packs all mutable module-level variables into a single LLVM
/// struct (`%ModuleName.Data`) and emits one `ModuleDesc` constant per module,
/// so the GC can scan module globals precisely without scanning arbitrary BSS
/// regions.
///
/// `ptrs` is a sentinel-terminated (first negative value) array of byte offsets
/// within `var_base` that identify heap-pointer fields.
#[repr(C)]
pub struct ModuleDesc {
    /// Pointer to the module's `%ModuleName.Data` global struct.
    pub var_base: *const u8,
    /// Sentinel-terminated array of byte offsets within `var_base` for pointer fields.
    pub ptrs: *const isize,
    /// Intrusive linked-list next pointer; null = end of list.
    pub next: *const ModuleDesc,
}

// ModuleDesc constants are emitted read-only by the compiler.
unsafe impl Sync for ModuleDesc {}
unsafe impl Send for ModuleDesc {}

// ─────────────────────────────────────────────────────────────────────────────
// Cluster — backing storage for managed allocations
// ─────────────────────────────────────────────────────────────────────────────

/// Default cluster size: 1 MiB. Tuned to amortise OS allocation overhead
/// while keeping conservative-scan walks bounded.
const DEFAULT_CLUSTER_SIZE: usize = 1 << 20;

/// All blocks (and the cluster base) are 16-byte aligned so payloads are safe
/// for any scalar, pointer, or common SIMD type.
const BLOCK_ALIGN: usize = 16;

/// Smallest legal block size (header + at least one payload word, 16-aligned).
/// Free-list splits will not produce a remainder smaller than this; otherwise
/// the slack stays attached to the allocated block.
const MIN_BLOCK: usize = 32;

/// Free-block payload prefix: a singly-linked list of free blocks within a
/// cluster. The link points at the **header** of the next free block.
#[repr(C)]
struct FreeBlockLink {
    next: *mut u8,
}

/// A single contiguous OS allocation that backs many managed blocks.
///
/// Layout: `[block_0][block_1]...[block_k][unused bump space]`
/// where `block_i` is either an allocated object or a free block (tag = 0)
/// linked into `free_list`. The `bump` cursor marks the boundary between
/// formed blocks and never-touched memory at the cluster's tail.
///
/// `block_starts` is a side-table bitmap (1 bit per `BLOCK_ALIGN`-aligned
/// position) where a set bit marks the start of a formed block. It enables
/// O(1)-amortised `resolve()` for the conservative stack scanner.
struct Cluster {
    /// Cluster base address (16-aligned).
    base: *mut u8,
    /// Cluster size in bytes.
    size: usize,
    /// Offset of the first byte past all formed blocks; `[0, bump)` is walkable.
    bump: usize,
    /// Head of this cluster's free-block linked list (header pointer or null).
    free_list: *mut u8,
    /// Layout used to allocate the cluster itself; required for eventual `dealloc`.
    layout: Layout,
    /// Block-start bitmap. Bit `i` is set iff a block begins at offset
    /// `i * BLOCK_ALIGN`. Sized to `cluster_size / BLOCK_ALIGN` bits.
    block_starts: Vec<u64>,
}

unsafe impl Send for Cluster {}

impl Drop for Cluster {
    fn drop(&mut self) {
        // Cluster backing memory is owned by the cluster; release it when the
        // cluster is dropped (e.g. if the GC ever shrinks the heap).
        unsafe { std::alloc::dealloc(self.base, self.layout) };
    }
}

impl Cluster {
    /// Allocates a fresh cluster large enough for `min_size` bytes of one block.
    fn new(min_size: usize) -> Self {
        let size = min_size.max(DEFAULT_CLUSTER_SIZE);
        let layout = Layout::from_size_align(size, BLOCK_ALIGN).unwrap();
        // alloc_zeroed gives every untouched payload byte a defined NIL/zero
        // value, satisfying CP's NEW semantics for the bump path for free.
        let base = unsafe { std::alloc::alloc_zeroed(layout) };
        if base.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        // One bit per BLOCK_ALIGN-aligned position; round up to whole u64s.
        let bits = size / BLOCK_ALIGN;
        let words = (bits + 63) / 64;
        Self {
            base,
            size,
            bump: 0,
            free_list: std::ptr::null_mut(),
            layout,
            block_starts: vec![0u64; words],
        }
    }

    /// Returns true if `addr` falls within the formed (walkable) region.
    #[inline]
    fn contains(&self, addr: usize) -> bool {
        let base = self.base as usize;
        addr >= base && addr < base + self.bump
    }

    /// Sets the block-start bit at the given byte offset within the cluster.
    #[inline]
    fn mark_block_start(&mut self, offset: usize) {
        debug_assert!(offset % BLOCK_ALIGN == 0);
        let bit = offset / BLOCK_ALIGN;
        self.block_starts[bit / 64] |= 1u64 << (bit % 64);
    }

    /// Clears the block-start bit at the given byte offset within the cluster.
    /// Used during sweep coalescing when an absorbed block is no longer a
    /// block-start of its own.
    #[inline]
    fn clear_block_start(&mut self, offset: usize) {
        debug_assert!(offset % BLOCK_ALIGN == 0);
        let bit = offset / BLOCK_ALIGN;
        self.block_starts[bit / 64] &= !(1u64 << (bit % 64));
    }

    /// Returns the byte offset of the largest block-start at or below `offset`,
    /// or `None` if no block starts at or below that position.
    ///
    /// Implementation: scan `block_starts` backwards from the word containing
    /// `offset`; in the first word, mask off bits above `offset`. Per-call
    /// cost is dominated by the leading-zero count on a single word for the
    /// common case where blocks are densely packed.
    fn block_start_at_or_below(&self, offset: usize) -> Option<usize> {
        if offset >= self.bump {
            // Cap to the last valid block-start position so we don't search
            // into never-formed bump space.
            return self.block_start_at_or_below(self.bump.saturating_sub(BLOCK_ALIGN));
        }
        let bit = offset / BLOCK_ALIGN;
        let word_idx = bit / 64;
        let bit_in_word = bit % 64;
        // Mask: bits 0..=bit_in_word inclusive.
        let mask = if bit_in_word == 63 {
            !0u64
        } else {
            (1u64 << (bit_in_word + 1)) - 1
        };
        let first = self.block_starts[word_idx] & mask;
        if first != 0 {
            let top = 63 - first.leading_zeros() as usize;
            return Some((word_idx * 64 + top) * BLOCK_ALIGN);
        }
        for w in (0..word_idx).rev() {
            let word = self.block_starts[w];
            if word != 0 {
                let top = 63 - word.leading_zeros() as usize;
                return Some((w * 64 + top) * BLOCK_ALIGN);
            }
        }
        None
    }

    /// Tries to satisfy a `total_size`-byte allocation from this cluster's
    /// free list, then from bump space. Returns the **header** pointer.
    /// Payload memory is zeroed before return.
    ///
    /// `total_size` must be a multiple of `BLOCK_ALIGN` and at least `MIN_BLOCK`.
    unsafe fn try_alloc(&mut self, total_size: usize) -> Option<*mut u8> {
        let header_size = std::mem::size_of::<BlockHeader>();

        // ── 1. Free-list (first-fit). ──────────────────────────────────────
        unsafe {
            let mut prev_link: *mut *mut u8 = &mut self.free_list;
            while !(*prev_link).is_null() {
                let block = *prev_link;
                let block_size = (*(block as *const BlockHeader)).block_size;
                let next_link = block.add(header_size) as *mut *mut u8;
                if block_size >= total_size {
                    // Unlink from the free list.
                    *prev_link = *next_link;

                    // Split if the leftover is itself a usable block.
                    let leftover = block_size - total_size;
                    if leftover >= MIN_BLOCK {
                        let split = block.add(total_size);
                        let split_offset =
                            (split as usize) - (self.base as usize);
                        let split_hdr = split as *mut BlockHeader;
                        (*split_hdr).tag = 0;
                        (*split_hdr).block_size = leftover;
                        let split_link =
                            split.add(header_size) as *mut FreeBlockLink;
                        split_link.write(FreeBlockLink { next: self.free_list });
                        self.free_list = split;
                        // Newly-formed split block becomes a block-start.
                        self.mark_block_start(split_offset);

                        (*(block as *mut BlockHeader)).block_size = total_size;
                    }
                    // Zero the payload (free-list path: payload may hold a stale link).
                    let final_size = (*(block as *const BlockHeader)).block_size;
                    let payload = block.add(header_size);
                    std::ptr::write_bytes(payload, 0, final_size - header_size);
                    return Some(block);
                }
                prev_link = next_link;
            }
        }

        // ── 2. Bump from cluster tail. ─────────────────────────────────────
        if self.bump.checked_add(total_size)? <= self.size {
            let block_offset = self.bump;
            let block = unsafe { self.base.add(block_offset) };
            self.bump += total_size;
            // Cluster memory was alloc_zeroed; payload is already zero.
            unsafe {
                let hdr = block as *mut BlockHeader;
                (*hdr).block_size = total_size;
                // tag is left for the caller to set (so callers don't depend
                // on a partially-initialised header during a window).
            }
            self.mark_block_start(block_offset);
            return Some(block);
        }

        None
    }

    /// Resolves an arbitrary address `addr` to the payload start of the block
    /// containing it, or `None` if `addr` is not inside an allocated block.
    /// Used by the conservative stack scanner.
    ///
    /// Uses the block-start bitmap for ~O(1) lookup instead of a linear walk.
    /// Free blocks return `None`.
    unsafe fn resolve(&self, addr: usize) -> Option<*const u8> {
        if !self.contains(addr) {
            return None;
        }
        let header_size = std::mem::size_of::<BlockHeader>();
        let base = self.base as usize;
        let offset_in_cluster = addr - base;
        let block_offset = self.block_start_at_or_below(offset_in_cluster)?;
        unsafe {
            let block = self.base.add(block_offset);
            let hdr = block as *const BlockHeader;
            let block_size = (*hdr).block_size;
            if block_size < MIN_BLOCK || block_offset + block_size > self.bump {
                return None; // corruption guard
            }
            let block_end = base + block_offset + block_size;
            if addr >= block_end {
                return None; // bitmap pointed to a block strictly before `addr`
            }
            let type_bits = (*hdr).tag & !BlockHeader::MARK_BIT;
            if type_bits == 0 {
                return None; // free block
            }
            let payload_start = base + block_offset + header_size;
            if addr < payload_start {
                return None; // address lands in the header region
            }
            Some(payload_start as *const u8)
        }
    }

    /// Sweeps this cluster: rebuilds the free list from unmarked blocks,
    /// invokes finalizers on newly-dead blocks, clears the mark bit on
    /// survivors, and coalesces adjacent free blocks.
    unsafe fn sweep(&mut self) {
        let header_size = std::mem::size_of::<BlockHeader>();
        self.free_list = std::ptr::null_mut();
        let mut offset: usize = 0;
        // The most recently produced free block in this pass, for coalescing.
        let mut prev_free: *mut u8 = std::ptr::null_mut();
        let mut prev_free_offset: usize = 0;

        unsafe {
            while offset < self.bump {
                let block = self.base.add(offset);
                let hdr = block as *mut BlockHeader;
                let block_size = (*hdr).block_size;
                if block_size < MIN_BLOCK || offset + block_size > self.bump {
                    // Corruption guard: stop walking this cluster.
                    break;
                }
                let raw_tag = (*hdr).tag;
                let type_bits = raw_tag & !BlockHeader::MARK_BIT;
                let is_marked = raw_tag & BlockHeader::MARK_BIT != 0;
                let was_free = type_bits == 0;
                let is_dead = !was_free && !is_marked;

                if !was_free && is_marked {
                    // Survivor: clear mark and continue.
                    (*hdr).clear_mark();
                    prev_free = std::ptr::null_mut();
                } else {
                    // Free block (newly dead, or already free).
                    if is_dead {
                        // Invoke finalizer (if any) before wiping payload.
                        let td = (type_bits) as *const TypeDesc;
                        if !td.is_null() {
                            if let Some(fin) = (*td).finalizer {
                                let payload = block.add(header_size);
                                fin(payload);
                            }
                        }
                        // Wipe the payload of newly-dead blocks so subsequent
                        // free-list traversals never observe stale GC pointers.
                        let payload = block.add(header_size);
                        std::ptr::write_bytes(payload, 0, block_size - header_size);
                    }

                    if !prev_free.is_null() {
                        // Coalesce with the immediately preceding free block:
                        // grow `prev_free` and clear this block's start bit.
                        let prev_hdr = prev_free as *mut BlockHeader;
                        (*prev_hdr).block_size += block_size;
                        self.clear_block_start(offset);
                        let _ = prev_free_offset; // keep variable alive for clarity
                    } else {
                        (*hdr).tag = 0;
                        (*hdr).block_size = block_size;
                        let link = block.add(header_size) as *mut FreeBlockLink;
                        link.write(FreeBlockLink { next: self.free_list });
                        self.free_list = block;
                        prev_free = block;
                        prev_free_offset = offset;
                    }
                }
                offset += block_size;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GcState — process-global GC bookkeeping
// ─────────────────────────────────────────────────────────────────────────────

/// Rust-owned per-module root record. Owns a copy of the offset array so the
/// GC never holds pointers into caller-managed memory.
struct ModuleRoots {
    /// Start address of the module's packed global data struct.
    var_base: *const u8,
    /// Owned copy of the byte-offset list for pointer fields within `var_base`.
    offsets: Vec<isize>,
}

// Access to `var_base` is always serialised through the `GC` mutex.
unsafe impl Send for ModuleRoots {}

struct GcState {
    /// Stack address recorded by `__newcp_init_gc`. The conservative scan
    /// walks upward from the current SP to this address.
    ///
    /// Follow-on action: If the runtime adds support for multiple concurrent
    /// managed threads, this must be decoupled from the global state and
    /// moved to a `thread_local!` cell.
    base_stack: usize,
    /// Identity of the thread that called `__newcp_init_gc`.
    ///
    /// **MVP-internal guard.** The current GC is single-threaded, and this
    /// field lets debug builds catch accidental cross-thread calls inside
    /// `gc.rs` only — it is *not* an architectural commitment exposed to other
    /// crates. Multi-thread evolution (per-thread `MutatorState`, cooperative
    /// safepoints) will replace this with a `thread_local!` registration
    /// scheme; see the "Multi-thread roadmap" section of
    /// `docs/garbage-collection.md`.
    /// `None` until the GC has been initialised.
    owner_thread: Option<ThreadId>,
    /// Backing storage for all managed allocations.
    clusters: Vec<Cluster>,
    /// All registered module root descriptors, in registration order.
    modules: Vec<ModuleRoots>,
}

/// Process-global GC state, protected by a mutex.
///
/// `Vec::new()` and `Mutex::new()` are both `const fn`, so this static is
/// fully constant-initialised without requiring a lazy initialiser.
static GC: Mutex<GcState> = Mutex::new(GcState {
    base_stack: 0,
    owner_thread: None,
    clusters: Vec::new(),
    modules: Vec::new(),
});

/// Debug-asserts that the calling thread is the GC's owning thread.
///
/// MVP-internal guard against accidental cross-thread calls while the GC is
/// single-threaded. Will be replaced by per-thread `MutatorState` registration
/// when stop-the-world cooperative safepoints land. Free in release builds.
#[inline]
fn debug_assert_owner_thread(gc: &GcState) {
    if cfg!(debug_assertions) {
        if let Some(owner) = gc.owner_thread {
            assert_eq!(
                owner,
                std::thread::current().id(),
                "GC entry point invoked from a non-owner thread; the MVP runtime is single-threaded \
                 (see docs/garbage-collection.md § Multi-thread roadmap)",
            );
        }
    }
}

/// Rounds `n` up to the next multiple of `BLOCK_ALIGN`.
#[inline]
fn align_up(n: usize) -> usize {
    (n + BLOCK_ALIGN - 1) & !(BLOCK_ALIGN - 1)
}

/// Computes the total block size (header + aligned payload, at least `MIN_BLOCK`)
/// required to satisfy a `payload_size` allocation request.
#[inline]
fn total_block_size(payload_size: usize) -> usize {
    let raw = std::mem::size_of::<BlockHeader>() + payload_size;
    align_up(raw).max(MIN_BLOCK)
}

/// Walks the cluster list and tries to allocate `total_size` bytes from any
/// existing cluster. Does **not** trigger collection or grow the heap.
unsafe fn try_alloc_in_clusters(gc: &mut GcState, total_size: usize) -> Option<*mut u8> {
    for cluster in &mut gc.clusters {
        if let Some(block) = unsafe { cluster.try_alloc(total_size) } {
            return Some(block);
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// JIT-callable exports
// ─────────────────────────────────────────────────────────────────────────────

/// Initialises the GC and records the stack base for conservative scanning.
///
/// Must be called once by the runtime startup on the **same thread** that will
/// execute Component Pascal code. If called a second time it is a no-op.
///
/// # Safety
/// `base_stack` must be the stack pointer at the boundary above all CP call
/// frames — i.e. the top of the "managed" stack region for this thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __newcp_init_gc(base_stack: *const u8) {
    let mut gc = GC.lock().unwrap();
    if gc.base_stack == 0 {
        gc.base_stack = base_stack as usize;
        gc.owner_thread = Some(std::thread::current().id());
    }
}

/// Registers a loaded module's global roots with the GC.
///
/// Called by the module loader after linking each module so that the GC will
/// trace the module's global pointer fields during the Mark phase.
///
/// An owned copy of the offset array is made immediately, so the caller does
/// not need to keep the original array live after returning.
///
/// # Safety
/// - `var_base` must point to the start of the module's live global data struct
///   and must remain valid for the lifetime of the module.
/// - `offsets_ptr` must point to a valid array of exactly `count` `isize` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __newcp_register_module(
    var_base: *const u8,
    offsets_ptr: *const isize,
    count: usize,
) {
    let offsets = unsafe { std::slice::from_raw_parts(offsets_ptr, count).to_vec() };
    let mut gc = GC.lock().unwrap();
    debug_assert_owner_thread(&gc);
    gc.modules.push(ModuleRoots { var_base, offsets });
}

/// Allocates and zero-initialises a new heap record.
///
/// **This is the *sole* heap-allocation entry point for managed code.**
/// All `NEW`, `NEW(..., len)` (array), and any future managed allocators
/// (closures, boxed primitives, …) must funnel through this function or a
/// thin wrapper that ultimately calls it. Other crates (loader, sema, codegen)
/// must never reach into `GcState` directly. Keeping a single chokepoint lets
/// us later swap in per-thread bump slabs (TLABs), allocation sampling, or a
/// safepoint poll without touching call sites.
///
/// Called directly by JIT-compiled `NEW` expressions. Returns a pointer to the
/// **payload** region (past the `BlockHeader`).
///
/// Allocation strategy (in order):
/// 1. Try free list, then bump pointer, in every existing cluster.
/// 2. If nothing fits, run a full `collect()` and retry.
/// 3. If still nothing fits, allocate a new cluster sized to fit at least this
///    object and retry.
///
/// CP `NEW` semantics require a zero-filled object; both the cluster bump path
/// (`alloc_zeroed`) and the free-list path (explicit `write_bytes`) satisfy this.
///
/// # Safety
/// - `tag` must point to a valid, live `TypeDesc` with a correct non-negative `size`.
/// - The `TypeDesc` (and its `ptroffs` array) must remain live for at least as
///   long as the returned allocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __newcp_new_rec(tag: *const TypeDesc) -> *mut u8 {
    // Spill registers/SP **outside** the lock so that an inline collect()
    // triggered below sees a correct stack range covering this frame.
    let mut spill_buf = [0usize; 16];
    let sp = capture_sp(&mut spill_buf);

    let payload_size = unsafe { (*tag).size as usize };
    let total_size = total_block_size(payload_size);
    let header_size = std::mem::size_of::<BlockHeader>();

    let mut gc = GC.lock().unwrap();
    debug_assert_owner_thread(&gc);

    // Step 1: try existing clusters.
    let block = unsafe { try_alloc_in_clusters(&mut gc, total_size) }
        .or_else(|| {
            // Step 2: allocation pressure → run a collection and retry.
            unsafe { collect_inner(&mut gc, sp) };
            unsafe { try_alloc_in_clusters(&mut gc, total_size) }
        })
        .unwrap_or_else(|| {
            // Step 3: heap growth.
            gc.clusters.push(Cluster::new(total_size));
            let last = gc.clusters.last_mut().expect("just pushed");
            unsafe { last.try_alloc(total_size) }
                .expect("fresh cluster must satisfy the request that drove its creation")
        });

    unsafe {
        let hdr = block as *mut BlockHeader;
        (*hdr).tag = tag as usize;
        (*hdr).block_size = total_size;
        block.add(header_size)
    }
}

/// Cooperative GC safepoint poll.
///
/// **Currently a no-op.** This entry point exists so that codegen can emit
/// safepoint polls now — at every loop back-edge and at function entry —
/// without paying any runtime cost today, and without a code-rewrite when
/// stop-the-world support lands.
///
/// Future implementation will:
/// 1. Load a global "GC requested" flag.
/// 2. If set, spill registers, mark this mutator parked, and block on a
///    condition variable until the GC cycle finishes.
///
/// Codegen contract: the call may clobber no registers beyond the C ABI's
/// caller-saved set, must be safe to invoke from any managed-code context,
/// and is allowed to block the calling thread for the duration of a GC cycle.
///
/// # Safety
/// Always safe to call. Marked `unsafe extern "C"` only for ABI symmetry with
/// the rest of the runtime exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __newcp_safepoint() {
    // Intentionally empty. See doc comment above.
}

// ─────────────────────────────────────────────────────────────────────────────
// Mark-phase helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Returns a mutable pointer to the `BlockHeader` preceding `payload`.
///
/// # Safety
/// `payload` must have been returned by `__newcp_new_rec`.
#[inline]
unsafe fn header_of(payload: *const u8) -> *mut BlockHeader {
    unsafe { payload.sub(std::mem::size_of::<BlockHeader>()) as *mut BlockHeader }
}

/// Resolves a suspected pointer to the payload start of the block containing
/// it, or `None` if the address is not in any allocated managed block.
///
/// Handles "interior pointers" (mid-array, mid-record) by returning the
/// containing block's payload start. Free blocks return `None`.
///
/// Per-cluster lookup is now bitmap-backed (~O(1) amortised). Cluster
/// selection is still a linear scan; with a small number of clusters
/// (typical), this is well under the cost of one stack-word probe.
/// Follow-on action: when cluster counts grow, replace the cluster scan with
/// an address-sorted index for O(log C) selection.
fn resolve_heap_ptr(addr: usize, gc: &GcState) -> Option<*const u8> {
    if addr == 0 {
        return None;
    }
    for cluster in &gc.clusters {
        if let Some(p) = unsafe { cluster.resolve(addr) } {
            return Some(p);
        }
    }
    None
}

/// Marks `start` and all objects transitively reachable through `TypeDesc.ptroffs`.
///
/// Uses an explicit work-stack (heap-allocated `Vec`) to avoid call-stack
/// overflow on deep or cyclic object graphs, as recommended by the design doc.
///
/// # Safety
/// `start` must point to the payload of a valid live GC allocation.
unsafe fn mark_object(start: *const u8) {
    // Pre-allocated capacity avoids the first few re-allocations for typical
    // object graphs.
    let mut work: Vec<*const u8> = Vec::with_capacity(64);
    work.push(start);

    while let Some(payload) = work.pop() {
        if payload.is_null() {
            continue;
        }
        unsafe {
            let hdr = header_of(payload);
            if (*hdr).is_marked() {
                continue; // already visited; cycle guard
            }
            // Defensive: never mark a free block. Conservative scanning
            // already filters these via resolve_heap_ptr, but precise roots
            // (modules, ptroffs children) trust their offsets blindly.
            let type_bits = (*hdr).tag & !BlockHeader::MARK_BIT;
            if type_bits == 0 {
                continue;
            }
            (*hdr).set_mark();

            let td = (*hdr).type_desc();
            if td.is_null() {
                continue;
            }

            // Enqueue all pointer-typed child fields.
            for offset in (*td).pointer_offsets() {
                let field = payload.add(offset as usize) as *const *const u8;
                let child = *field;
                if !child.is_null() {
                    work.push(child);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Register spill + stack-pointer capture
// ─────────────────────────────────────────────────────────────────────────────

/// Writes callee-saved registers into `spill_buf` (which lives in the caller's
/// stack frame) and returns the current stack pointer.
///
/// The caller declares `spill_buf` as a local so that:
/// 1. `spill_buf` is forced onto the stack (taking `&mut` precludes registers).
/// 2. The stack scan starting at the returned SP will cover `spill_buf` and
///    thus see any live GC roots that were held only in registers at call time.
///
/// `#[inline(never)]` is required: inlining would merge frames and the captured
/// SP would be incorrect.
#[inline(never)]
fn capture_sp(spill_buf: &mut [usize; 16]) -> usize {
    let sp: usize;
    unsafe {
        #[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
        std::arch::asm!(
            // Spill all callee-saved GP registers (System-V ABI: rbx, rbp, r12-r15).
            "mov [{buf}     ], rbx",
            "mov [{buf} +  8], rbp",
            "mov [{buf} + 16], r12",
            "mov [{buf} + 24], r13",
            "mov [{buf} + 32], r14",
            "mov [{buf} + 40], r15",
            // Capture RSP after the spills so the scan starts below them.
            "mov {sp}, rsp",
            buf = in(reg) spill_buf.as_mut_ptr(),
            sp  = out(reg) sp,
        );

        #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
        std::arch::asm!(
            // Spill Windows x64 ABI callee-saved registers (rbx, rbp, rdi, rsi, r12-r15).
            "mov [{buf}     ], rbx",
            "mov [{buf} +  8], rbp",
            "mov [{buf} + 16], rdi",
            "mov [{buf} + 24], rsi",
            "mov [{buf} + 32], r12",
            "mov [{buf} + 40], r13",
            "mov [{buf} + 48], r14",
            "mov [{buf} + 56], r15",
            // Note: XMM6-XMM15 are also callee-saved on Windows but generally not used 
            // for managed heap pointers. A full conservative scan might also need to spill those.
            "mov {sp}, rsp",
            buf = in(reg) spill_buf.as_mut_ptr(),
            sp  = out(reg) sp,
        );

        #[cfg(target_arch = "aarch64")]
        std::arch::asm!(
            // Spill callee-saved GP registers (AArch64 AAPCS: x19-x28).
            "stp x19, x20, [{buf}]",
            "stp x21, x22, [{buf}, #16]",
            "stp x23, x24, [{buf}, #32]",
            "stp x25, x26, [{buf}, #48]",
            "mov {sp}, sp",
            buf = in(reg) spill_buf.as_mut_ptr(),
            sp  = out(reg) sp,
        );

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Fallback: address of the spill buffer is a safe SP lower bound.
            // Callee-saved register contents may not be captured here; this
            // path is acceptable for development / non-production targets.
            sp = spill_buf.as_ptr() as usize;
        }
    }
    sp
}

// ─────────────────────────────────────────────────────────────────────────────
// collect()
// ─────────────────────────────────────────────────────────────────────────────

/// Triggers a full Mark-and-Sweep garbage collection cycle.
///
/// This is the public entry point invoked by:
/// - explicit `Kernel.Collect()` calls from Component Pascal code,
/// - the runtime's idle loop (when present),
/// - the test harness.
///
/// Allocation pressure inside `__newcp_new_rec` invokes `collect_inner`
/// directly to avoid releasing and re-acquiring the GC lock.
///
/// ## Phase 1 — Mark
/// 1. Spill callee-saved registers to a stack buffer in this frame.
/// 2. Clear all existing mark bits.
/// 3. Conservatively scan the stack from the captured SP up to `base_stack`:
///    any word whose value resolves to a managed block payload is treated as
///    a root and marked.
/// 4. Precisely scan all registered module global roots via their offset lists.
/// 5. Transitively trace all marked objects through `TypeDesc.ptroffs` using
///    an explicit work-stack (no recursion, no stack-overflow risk).
///
/// ## Phase 2 — Sweep
/// Walk every cluster linearly. Unmarked blocks become free-list entries
/// (with adjacent runs coalesced); marked blocks have their mark bit cleared
/// for the next cycle.
pub fn collect() {
    // Spill registers into a local buffer **before** locking, so that live
    // pointer values held in callee-saved registers land on the stack and are
    // visible to the conservative scan below.
    let mut spill_buf = [0usize; 16];
    let sp = capture_sp(&mut spill_buf);

    let mut gc = GC.lock().unwrap();
    debug_assert_owner_thread(&gc);
    unsafe { collect_inner(&mut gc, sp) };
}

/// Mark-and-sweep against an already-locked `GcState` using a pre-captured `sp`.
///
/// Used both by `collect()` (public entry point) and by `__newcp_new_rec`
/// when allocation pressure forces a cycle without releasing the GC lock.
///
/// # Safety
/// - `sp` must be a valid stack pointer captured in a frame at least as deep
///   as the current frame (i.e. `sp` ≤ this frame's SP) so the scanned range
///   covers all live managed roots.
/// - Caller must hold the `GC` mutex (`gc` is the locked `MutexGuard`'s `&mut`).
unsafe fn collect_inner(gc: &mut GcState, sp: usize) {
    // ── Phase 1a: Clear all mark bits across every cluster ───────────────────
    unsafe {
        for cluster in &mut gc.clusters {
            let header_size = std::mem::size_of::<BlockHeader>();
            let mut offset: usize = 0;
            while offset < cluster.bump {
                let hdr = cluster.base.add(offset) as *mut BlockHeader;
                let block_size = (*hdr).block_size;
                if block_size < MIN_BLOCK || offset + block_size > cluster.bump {
                    break; // corruption guard
                }
                (*hdr).clear_mark();
                offset += block_size;
                let _ = header_size; // silence unused warning if compiler reorders
            }
        }
    }

    let base_stack = gc.base_stack;

    // ── Phase 1b: Conservative stack scan ────────────────────────────────────
    // Stack grows downward: SP is the lowest live address, base_stack the highest.
    // Scan word-by-word from SP upward. The `spill_buf` declared by the caller
    // lives below this frame and is covered by this scan.
    if base_stack != 0 && sp < base_stack {
        let word = std::mem::size_of::<usize>();
        let mut cursor = sp;
        while cursor < base_stack {
            let val = unsafe { *(cursor as *const usize) };
            if let Some(payload_base) = resolve_heap_ptr(val, gc) {
                unsafe { mark_object(payload_base) };
            }
            cursor += word;
        }
    }

    // ── Phase 1c: Precise global / module roots ───────────────────────────────
    unsafe {
        for module in &gc.modules {
            for &offset in &module.offsets {
                let field = module.var_base.add(offset as usize) as *const *const u8;
                let ptr = *field;
                if !ptr.is_null() {
                    mark_object(ptr);
                }
            }
        }
    }

    // ── Phase 2: Sweep every cluster ─────────────────────────────────────────
    // Each cluster's sweep rebuilds its own free list and coalesces runs of
    // adjacent free blocks. Blocks themselves are reclaimed in place; cluster
    // memory is never released back to the OS in the current implementation.
    //
    // Follow-on action: release fully-empty clusters (all blocks free,
    // coalesced into a single span equal to `bump`) back to the OS once a
    // policy is decided.
    unsafe {
        for cluster in &mut gc.clusters {
            cluster.sweep();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests run serially because they share the process-global `GC` state.
    /// We just acquire the mutex around the whole scenario; tests panic on
    /// failure rather than returning an `Err`.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Acquire the test lock, recovering from poisoning so a single failing
    /// test doesn't cascade-poison every subsequent test in this module.
    fn lock_tests() -> std::sync::MutexGuard<'static, ()> {
        match TEST_LOCK.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        }
    }

    /// Wipes the global GC state to a known-empty baseline so each test
    /// starts from the same point.
    fn reset_gc() {
        let mut gc = GC.lock().unwrap();
        // Drop all clusters (and their backing memory) and clear all roots.
        gc.clusters.clear();
        gc.modules.clear();
        gc.base_stack = 0;
        gc.owner_thread = None;
    }

    /// Builds a `TypeDesc` with no pointer fields, suitable for objects of
    /// `payload_size` bytes. Returned as a leaked `Box` so the address stays
    /// valid for the lifetime of the test.
    fn make_leaf_type_desc(payload_size: usize) -> *const TypeDesc {
        // 1 entry for the `-1` sentinel.
        #[repr(C)]
        struct LeafTd {
            base: TypeDesc,
            sentinel: isize,
        }
        let td = Box::new(LeafTd {
            base: TypeDesc {
                size: payload_size as isize,
                module: std::ptr::null(),
                finalizer: None,
                ptroffs: [],
            },
            sentinel: -1,
        });
        // Leak so the TypeDesc lives for the rest of the test (and process).
        let raw = Box::into_raw(td);
        raw as *const TypeDesc
    }

    /// Builds a `TypeDesc` with the given pointer-field byte offsets and an
    /// optional finalizer. Returned as a leaked `Box`.
    fn make_type_desc_with_ptrs(
        payload_size: usize,
        ptr_offsets: &[isize],
        finalizer: Option<Finalizer>,
    ) -> *const TypeDesc {
        // Tail layout: [ptr_offsets...][-1 sentinel].
        let total = std::mem::size_of::<TypeDesc>()
            + (ptr_offsets.len() + 1) * std::mem::size_of::<isize>();
        let layout = std::alloc::Layout::from_size_align(total, std::mem::align_of::<TypeDesc>())
            .unwrap();
        unsafe {
            let raw = std::alloc::alloc(layout) as *mut TypeDesc;
            assert!(!raw.is_null());
            (*raw).size = payload_size as isize;
            (*raw).module = std::ptr::null();
            (*raw).finalizer = finalizer;
            // Write the trailing array immediately past the struct.
            let tail =
                (raw as *mut u8).add(std::mem::size_of::<TypeDesc>()) as *mut isize;
            for (i, &off) in ptr_offsets.iter().enumerate() {
                tail.add(i).write(off);
            }
            tail.add(ptr_offsets.len()).write(-1);
            // Layout is leaked; tests don't free TypeDescs.
            raw as *const TypeDesc
        }
    }

    #[test]
    fn alloc_zero_initialised() {
        let _t = lock_tests();
        reset_gc();

        let td = make_leaf_type_desc(64);
        unsafe {
            let p = __newcp_new_rec(td);
            assert!(!p.is_null());
            // Every byte of payload must be zero (CP NEW semantics).
            for i in 0..64 {
                assert_eq!(*p.add(i), 0, "payload byte {i} not zero");
            }
        }
    }

    #[test]
    fn growth_then_collection_reclaims() {
        let _t = lock_tests();
        reset_gc();

        // Initialise a base_stack at the end of a known-mapped local buffer
        // so the conservative scan never reads past the OS stack mapping.
        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let td = make_leaf_type_desc(64);

        // Allocate enough to definitely exceed the bump space of one cluster.
        // 1 MiB cluster / ~96B per block = ~10 920 blocks; we do far less.
        let mut allocated_count = 0usize;
        for _ in 0..2_000 {
            let p = unsafe { __newcp_new_rec(td) };
            assert!(!p.is_null());
            allocated_count += 1;
            // Deliberately drop p — no root references survive past this loop.
        }
        assert_eq!(allocated_count, 2_000);

        // Snapshot bump usage before collect.
        let bump_before: usize = {
            let gc = GC.lock().unwrap();
            gc.clusters.iter().map(|c| c.bump).sum()
        };

        // Force a collection. With no live roots, every block must end up free.
        collect();

        // After sweep, free lists should hold all of the previously allocated
        // space (coalesced). Re-allocating should now succeed without growing
        // bump (i.e. without touching new tail memory).
        let (bump_after, free_count_after): (usize, usize) = {
            let gc = GC.lock().unwrap();
            let bump: usize = gc.clusters.iter().map(|c| c.bump).sum();
            let free: usize = gc
                .clusters
                .iter()
                .map(|c| {
                    let mut n = 0usize;
                    let mut cur = c.free_list;
                    while !cur.is_null() {
                        n += 1;
                        let header_size = std::mem::size_of::<BlockHeader>();
                        unsafe {
                            let link = cur.add(header_size) as *const FreeBlockLink;
                            cur = (*link).next;
                        }
                    }
                    n
                })
                .sum();
            (bump, free)
        };
        assert_eq!(
            bump_before, bump_after,
            "sweep must not change bump cursor"
        );
        assert!(
            free_count_after > 0,
            "sweep should produce at least one free block per cluster with live garbage"
        );
    }

    #[test]
    fn freed_blocks_are_reused() {
        let _t = lock_tests();
        reset_gc();

        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let td = make_leaf_type_desc(64);

        // Allocate, drop refs, collect → free space available.
        for _ in 0..50 {
            let _ = unsafe { __newcp_new_rec(td) };
        }
        let bump_before = {
            let gc = GC.lock().unwrap();
            gc.clusters.iter().map(|c| c.bump).sum::<usize>()
        };
        collect();

        // Allocate again: the bump cursor should not advance, because every
        // request is satisfied from the free list.
        for _ in 0..50 {
            let _ = unsafe { __newcp_new_rec(td) };
        }
        let bump_after = {
            let gc = GC.lock().unwrap();
            gc.clusters.iter().map(|c| c.bump).sum::<usize>()
        };
        assert_eq!(
            bump_before, bump_after,
            "free list must be exhausted before bump grows again"
        );
    }

    #[test]
    fn module_root_keeps_object_alive() {
        let _t = lock_tests();
        reset_gc();

        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let td = make_leaf_type_desc(64);

        // Allocate one object and store its payload pointer in a "module
        // global" — a static slot at offset 0 of a global data struct.
        // Use a Box<*mut u8> as the module's var_base; offset 0 is a pointer.
        let slot: Box<*mut u8> = Box::new(std::ptr::null_mut());
        let slot_ptr = Box::into_raw(slot);

        let payload = unsafe { __newcp_new_rec(td) };
        unsafe { slot_ptr.write(payload) };

        // Register the module: var_base = &slot, ptr offset 0.
        let offsets = [0isize];
        unsafe {
            __newcp_register_module(slot_ptr as *const u8, offsets.as_ptr(), offsets.len())
        };

        // Allocate a bunch of garbage and collect; the rooted object must survive.
        for _ in 0..200 {
            let _ = unsafe { __newcp_new_rec(td) };
        }
        collect();

        // The rooted object's header must still be marked-clear and allocated
        // (tag != 0 with mark bit cleared by the cycle).
        let header_size = std::mem::size_of::<BlockHeader>();
        unsafe {
            let hdr = payload.sub(header_size) as *const BlockHeader;
            let type_bits = (*hdr).tag & !BlockHeader::MARK_BIT;
            assert_ne!(type_bits, 0, "rooted object was reclaimed by sweep");
            assert_eq!(type_bits, td as usize, "rooted object's tag changed");
        }

        // Cleanup the module registration so other tests see a clean slate.
        // (reset_gc in the next test will drop modules anyway, but free the
        // slot now to avoid leaking it across the test process.)
        unsafe { drop(Box::from_raw(slot_ptr)) };
    }

    // ─────────────────────────────────────────────────────────────────────
    // Stress / correctness tests for graph-shaped object reachability.
    // ─────────────────────────────────────────────────────────────────────

    /// Counts how many allocated (non-free, non-marked-bit-set) blocks of
    /// type `td` exist across all clusters. Used by stress tests to verify
    /// reclamation without depending on free-list shape.
    fn live_block_count(td: *const TypeDesc) -> usize {
        let gc = GC.lock().unwrap();
        let mut n = 0usize;
        for cluster in &gc.clusters {
            let mut offset = 0usize;
            unsafe {
                while offset < cluster.bump {
                    let hdr = cluster.base.add(offset) as *const BlockHeader;
                    let block_size = (*hdr).block_size;
                    if block_size < MIN_BLOCK || offset + block_size > cluster.bump {
                        break;
                    }
                    let type_bits = (*hdr).tag & !BlockHeader::MARK_BIT;
                    if type_bits == td as usize {
                        n += 1;
                    }
                    offset += block_size;
                }
            }
        }
        n
    }

    /// Cycle reclamation: A → B → A with no external root must collect both.
    ///
    /// We deliberately skip `__newcp_init_gc` so the conservative stack scan
    /// is disabled (`base_stack == 0`). This isolates the test from stale
    /// heap-pointer-looking words left behind in popped stack frames, which
    /// would otherwise non-deterministically pin garbage. Conservative stack
    /// scanning itself is exercised by the earlier tests.
    #[test]
    fn cyclic_garbage_is_reclaimed() {
        let _t = lock_tests();
        reset_gc();
        // Note: no __newcp_init_gc → no stack scan.

        // Single pointer field at offset 0; payload = one usize.
        let td = make_type_desc_with_ptrs(
            std::mem::size_of::<usize>(),
            &[0],
            None,
        );

        // Build A ↔ B inside a separate function so that, after it returns,
        // the entire frame holding the local pointers is popped — the
        // conservative scanner only walks SP upward to base_stack and so
        // cannot see the popped slots.
        #[inline(never)]
        fn build_cycle(td: *const TypeDesc) {
            unsafe {
                let a = __newcp_new_rec(td);
                let b = __newcp_new_rec(td);
                (a as *mut *mut u8).write(b);
                (b as *mut *mut u8).write(a);
            }
        }
        build_cycle(td);

        collect();

        assert_eq!(
            live_block_count(td),
            0,
            "unrooted A↔B cycle must be reclaimed"
        );
    }

    /// Deep linked list rooted via a module global: must survive collection,
    /// and the marker must not stack-overflow on long chains. After unrooting,
    /// every node must be reclaimed.
    ///
    /// Skips `__newcp_init_gc` (no stack scan) for deterministic reclamation.
    #[test]
    fn deep_linked_list_survives_then_reclaims() {
        let _t = lock_tests();
        reset_gc();
        // Note: no __newcp_init_gc → no stack scan.

        // List node = { next: *mut u8 } at offset 0.
        let td = make_type_desc_with_ptrs(
            std::mem::size_of::<usize>(),
            &[0],
            None,
        );

        const N: usize = 10_000;

        // Module root = a single pointer slot.
        let slot: Box<*mut u8> = Box::new(std::ptr::null_mut());
        let slot_ptr = Box::into_raw(slot);
        let offsets = [0isize];
        unsafe {
            __newcp_register_module(
                slot_ptr as *const u8,
                offsets.as_ptr(),
                offsets.len(),
            );
        }

        // Build N-node list, head in the module slot. Construction happens
        // in a helper so its frame is popped before any collect; otherwise
        // the conservative scanner would still see the local `head` slot.
        #[inline(never)]
        fn build_list(td: *const TypeDesc, slot_ptr: *mut *mut u8, n: usize) {
            let mut head: *mut u8 = std::ptr::null_mut();
            for _ in 0..n {
                let node = unsafe { __newcp_new_rec(td) };
                unsafe { (node as *mut *mut u8).write(head) };
                head = node;
            }
            unsafe { slot_ptr.write(head) };
        }
        build_list(td, slot_ptr, N);

        // Survive across collection.
        collect();
        assert_eq!(
            live_block_count(td),
            N,
            "all N rooted list nodes must survive"
        );

        // Drop the root and collect: every node must die. Null the slot from
        // a helper to ensure no stale local copies of `head` survive on the
        // stack into collect().
        #[inline(never)]
        fn clear_slot(slot_ptr: *mut *mut u8) {
            unsafe { slot_ptr.write(std::ptr::null_mut()) };
        }
        clear_slot(slot_ptr);
        collect();
        assert_eq!(
            live_block_count(td),
            0,
            "all N nodes must be reclaimed once unrooted"
        );

        unsafe { drop(Box::from_raw(slot_ptr)) };
    }

    /// Record with many pointer fields: tracing must enqueue every child.
    #[test]
    fn wide_pointer_record_traces_all_children() {
        let _t = lock_tests();
        reset_gc();
        // No stack scan: this test asserts exact survivor counts driven by
        // the precise module-root + tracing path.

        const FIELDS: usize = 64;
        let payload_size = FIELDS * std::mem::size_of::<usize>();
        let offsets: Vec<isize> = (0..FIELDS)
            .map(|i| (i * std::mem::size_of::<usize>()) as isize)
            .collect();
        let parent_td = make_type_desc_with_ptrs(payload_size, &offsets, None);
        let leaf_td = make_leaf_type_desc(32);

        let slot: Box<*mut u8> = Box::new(std::ptr::null_mut());
        let slot_ptr = Box::into_raw(slot);
        let mod_offsets = [0isize];
        unsafe {
            __newcp_register_module(
                slot_ptr as *const u8,
                mod_offsets.as_ptr(),
                mod_offsets.len(),
            );
        }

        // Allocate parent + FIELDS leaves; wire them up. Done in a helper so
        // the loop variables and `parent` local don't leak into the scanned
        // stack region across collect().
        #[inline(never)]
        fn wire_parent(
            parent_td: *const TypeDesc,
            leaf_td: *const TypeDesc,
            slot_ptr: *mut *mut u8,
            fields: usize,
        ) {
            let parent = unsafe { __newcp_new_rec(parent_td) };
            for i in 0..fields {
                let leaf = unsafe { __newcp_new_rec(leaf_td) };
                unsafe {
                    let field = (parent as *mut *mut u8).add(i);
                    field.write(leaf);
                }
            }
            unsafe { slot_ptr.write(parent) };
        }
        wire_parent(parent_td, leaf_td, slot_ptr, FIELDS);

        // Add unrooted garbage.
        #[inline(never)]
        fn allocate_garbage(leaf_td: *const TypeDesc, n: usize) {
            for _ in 0..n {
                let _ = unsafe { __newcp_new_rec(leaf_td) };
            }
        }
        allocate_garbage(leaf_td, 200);

        collect();

        assert_eq!(live_block_count(parent_td), 1, "parent must survive");
        assert_eq!(
            live_block_count(leaf_td),
            FIELDS,
            "exactly the {FIELDS} child leaves must survive"
        );

        unsafe { drop(Box::from_raw(slot_ptr)) };
    }

    /// Mixed allocation sizes: exercises free-list split + coalesce paths.
    #[test]
    fn mixed_size_alloc_free_cycle() {
        let _t = lock_tests();
        reset_gc();
        // No stack scan: deterministic full-reclamation expectation.

        let small = make_leaf_type_desc(16);
        let medium = make_leaf_type_desc(96);
        let large = make_leaf_type_desc(512);

        // Allocate inside a helper so the per-iteration locals are popped.
        #[inline(never)]
        fn allocate_garbage(
            small: *const TypeDesc,
            medium: *const TypeDesc,
            large: *const TypeDesc,
            n: usize,
        ) {
            for _ in 0..n {
                let _ = unsafe { __newcp_new_rec(small) };
                let _ = unsafe { __newcp_new_rec(medium) };
                let _ = unsafe { __newcp_new_rec(large) };
            }
        }
        allocate_garbage(small, medium, large, 500);

        let bump_before: usize = {
            let gc = GC.lock().unwrap();
            gc.clusters.iter().map(|c| c.bump).sum()
        };

        collect();

        assert_eq!(live_block_count(small), 0);
        assert_eq!(live_block_count(medium), 0);
        assert_eq!(live_block_count(large), 0);

        // After full reclamation, allocate a mix again — bump must not advance
        // (free list satisfies all requests).
        #[inline(never)]
        fn allocate_more(small: *const TypeDesc, large: *const TypeDesc, n: usize) {
            for _ in 0..n {
                let _ = unsafe { __newcp_new_rec(small) };
                let _ = unsafe { __newcp_new_rec(large) };
            }
        }
        allocate_more(small, large, 100);
        let bump_after: usize = {
            let gc = GC.lock().unwrap();
            gc.clusters.iter().map(|c| c.bump).sum()
        };
        assert_eq!(
            bump_before, bump_after,
            "reused free space must not push the bump cursor"
        );
    }

    /// Finalizers fire exactly once per dying block, before payload zeroing.
    #[test]
    fn finalizer_runs_once_on_dead_blocks() {
        let _t = lock_tests();
        reset_gc();
        // No stack scan: deterministic finalizer count.

        // A finalizer that increments a global counter.
        static FIN_COUNT: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        unsafe extern "C" fn fin(_payload: *mut u8) {
            FIN_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        FIN_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);

        let td = make_type_desc_with_ptrs(64, &[], Some(fin));
        const N: usize = 100;
        #[inline(never)]
        fn allocate(td: *const TypeDesc, n: usize) {
            for _ in 0..n {
                let _ = unsafe { __newcp_new_rec(td) };
            }
        }
        allocate(td, N);

        collect();

        assert_eq!(
            FIN_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            N,
            "finalizer must fire exactly once per dead block"
        );

        // A second collection must not re-fire finalizers on already-free blocks.
        collect();
        assert_eq!(
            FIN_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            N,
            "finalizer must not run again on already-free blocks"
        );
    }
}