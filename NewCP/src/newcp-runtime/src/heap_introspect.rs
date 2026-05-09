//! Read-only heap introspection — counters, snapshots, and the text/JSON
//! report used by `dump-heap`.
//!
//! See `docs/heap_introspection.md` for the design rationale. Three layers:
//! 1. Always-on atomic counters (cheap, lock-free reads).
//! 2. Snapshot of clusters + module roots (locked, owned-data return).
//! 3. Per-block walk + type catalog (locked, walks every formed block).

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::gc::{self, BlockHeader, Cluster, GcState, ModuleRoots, TypeDesc, HEAP_COUNTERS};

// ─────────────────────────────────────────────────────────────────────────────
// Counters — Layer 1
// ─────────────────────────────────────────────────────────────────────────────

/// Lock-free snapshot of the always-on heap counters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeapCountersSnapshot {
    pub alloc_blocks_lifetime: u64,
    pub alloc_bytes_lifetime: u64,
    pub free_blocks_lifetime: u64,
    pub free_bytes_lifetime: u64,
    pub bump_path_blocks: u64,
    pub free_list_path_blocks: u64,
    pub grow_events: u64,
    pub collect_cycles: u64,
    pub collect_total_nanos: u64,
    pub collect_last_nanos: u64,
    pub collect_last_reclaimed_bytes: u64,
    pub live_blocks: u64,
    pub live_bytes: u64,
    pub cluster_count: u64,
    pub module_root_count: u64,
    pub peak_live_bytes: u64,
}

/// Atomically copy every counter into an owned snapshot.
///
/// Reads are `Relaxed` for the lifetime totals and `Acquire` for fields that
/// pair with `AcqRel` writes (`collect_total_nanos`, `peak_live_bytes`).
pub fn current_counters() -> HeapCountersSnapshot {
    HeapCountersSnapshot {
        alloc_blocks_lifetime: HEAP_COUNTERS
            .alloc_blocks_lifetime
            .load(Ordering::Relaxed),
        alloc_bytes_lifetime: HEAP_COUNTERS
            .alloc_bytes_lifetime
            .load(Ordering::Relaxed),
        free_blocks_lifetime: HEAP_COUNTERS
            .free_blocks_lifetime
            .load(Ordering::Relaxed),
        free_bytes_lifetime: HEAP_COUNTERS.free_bytes_lifetime.load(Ordering::Relaxed),
        bump_path_blocks: HEAP_COUNTERS.bump_path_blocks.load(Ordering::Relaxed),
        free_list_path_blocks: HEAP_COUNTERS
            .free_list_path_blocks
            .load(Ordering::Relaxed),
        grow_events: HEAP_COUNTERS.grow_events.load(Ordering::Relaxed),
        collect_cycles: HEAP_COUNTERS.collect_cycles.load(Ordering::Relaxed),
        collect_total_nanos: HEAP_COUNTERS
            .collect_total_nanos
            .load(Ordering::Acquire),
        collect_last_nanos: HEAP_COUNTERS.collect_last_nanos.load(Ordering::Relaxed),
        collect_last_reclaimed_bytes: HEAP_COUNTERS
            .collect_last_reclaimed_bytes
            .load(Ordering::Relaxed),
        live_blocks: HEAP_COUNTERS.live_blocks.load(Ordering::Relaxed),
        live_bytes: HEAP_COUNTERS.live_bytes.load(Ordering::Relaxed),
        cluster_count: HEAP_COUNTERS.cluster_count.load(Ordering::Relaxed),
        module_root_count: HEAP_COUNTERS.module_root_count.load(Ordering::Relaxed),
        peak_live_bytes: HEAP_COUNTERS.peak_live_bytes.load(Ordering::Acquire),
    }
}

/// Reset every atomic counter to zero. Test / `dump-heap --reset` helper.
/// Not safe to call mid-run if anyone else is reading the counters.
pub fn reset_counters() {
    HEAP_COUNTERS.reset();
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-cluster + per-module snapshot — Layer 2 (lite) and 3 (full)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClusterSnapshot {
    pub index: usize,
    /// Numeric address of the cluster base. Surfaced for diagnostics; never
    /// dereferenced by consumers.
    pub base: usize,
    pub size: usize,
    pub bump: usize,
    pub live_blocks: u64,
    pub live_bytes: u64,
    pub free_blocks: u64,
    pub free_bytes: u64,
    pub largest_free_block: u64,
    /// 0.0 = no fragmentation (one big free span) or no free space at all.
    /// 1.0 = pathological (all free space split into many tiny pieces).
    pub fragmentation_ratio: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleRootSnapshot {
    pub name: String,
    /// Numeric address of `var_base`. Diagnostics only.
    pub var_base: usize,
    pub offsets: Vec<isize>,
    /// `*((var_base + offsets[i]) as *const usize)` captured at snapshot time.
    /// May contain stale or zero values for slots that aren't yet initialised.
    pub current_pointer_values: Vec<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeSnapshot {
    pub display_name: String,
    pub type_desc_addr: usize,
    pub size: isize,
    pub vtable_len: u64,
    pub has_finalizer: bool,
    pub instance_count: u64,
    pub instance_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeapSnapshot {
    pub taken_at_ns_since_epoch: u128,
    pub counters: HeapCountersSnapshot,
    pub clusters: Vec<ClusterSnapshot>,
    pub modules: Vec<ModuleRootSnapshot>,
    pub types: Vec<TypeSnapshot>,
}

/// Counters + cluster/module headline only, no per-block walk.
///
/// Cost is O(clusters + modules). Suitable for live status displays. The
/// per-cluster `live_blocks` / `free_blocks` fields are zero in lite mode —
/// use [`take_snapshot`] when those numbers matter.
pub fn take_lite_snapshot() -> HeapSnapshot {
    let counters = current_counters();
    let (clusters, modules) = gc::with_locked_state(|state| {
        (snapshot_clusters_lite(state), snapshot_modules(state))
    });
    HeapSnapshot {
        taken_at_ns_since_epoch: now_ns(),
        counters,
        clusters,
        modules,
        types: Vec::new(),
    }
}

/// Full snapshot: per-cluster occupancy and a type catalog built by walking
/// every formed block in every cluster. Cost is O(total blocks). Locks the
/// GC for the duration of the walk.
pub fn take_snapshot() -> HeapSnapshot {
    let counters = current_counters();
    let (clusters, modules, types) = gc::with_locked_state(|state| {
        (
            snapshot_clusters_full(state),
            snapshot_modules(state),
            snapshot_types(state),
        )
    });
    HeapSnapshot {
        taken_at_ns_since_epoch: now_ns(),
        counters,
        clusters,
        modules,
        types,
    }
}

fn snapshot_clusters_lite(state: &GcState) -> Vec<ClusterSnapshot> {
    state
        .clusters
        .iter()
        .enumerate()
        .map(|(index, cluster)| ClusterSnapshot {
            index,
            base: cluster.base as usize,
            size: cluster.size,
            bump: cluster.bump,
            ..Default::default()
        })
        .collect()
}

fn snapshot_clusters_full(state: &GcState) -> Vec<ClusterSnapshot> {
    state
        .clusters
        .iter()
        .enumerate()
        .map(|(index, cluster)| {
            let stats = walk_cluster_occupancy(cluster);
            let frag = if stats.free_bytes > 0 && stats.free_blocks > 1 {
                1.0 - (stats.largest_free_block as f32 / stats.free_bytes as f32)
            } else {
                0.0
            };
            ClusterSnapshot {
                index,
                base: cluster.base as usize,
                size: cluster.size,
                bump: cluster.bump,
                live_blocks: stats.live_blocks,
                live_bytes: stats.live_bytes,
                free_blocks: stats.free_blocks,
                free_bytes: stats.free_bytes,
                largest_free_block: stats.largest_free_block,
                fragmentation_ratio: frag,
            }
        })
        .collect()
}

fn snapshot_modules(state: &GcState) -> Vec<ModuleRootSnapshot> {
    state
        .modules
        .iter()
        .map(|m| {
            let pointer_values: Vec<usize> = m
                .offsets
                .iter()
                .map(|&offset| unsafe {
                    let field = m.var_base.add(offset as usize) as *const usize;
                    *field
                })
                .collect();
            ModuleRootSnapshot {
                name: m.name.clone(),
                var_base: m.var_base as usize,
                offsets: m.offsets.clone(),
                current_pointer_values: pointer_values,
            }
        })
        .collect()
}

fn snapshot_types(state: &GcState) -> Vec<TypeSnapshot> {
    let header_size = std::mem::size_of::<BlockHeader>();
    let mut buckets: HashMap<usize, TypeBucket> = HashMap::new();

    for cluster in &state.clusters {
        let mut offset = 0usize;
        unsafe {
            while offset < cluster.bump {
                let block = cluster.base.add(offset);
                let hdr = block as *const BlockHeader;
                let block_size = (*hdr).block_size;
                if block_size < 32 || offset + block_size > cluster.bump {
                    break;
                }
                let raw_tag = (*hdr).tag;
                let type_bits = raw_tag & !1usize;
                if type_bits != 0 {
                    let entry = buckets.entry(type_bits).or_default();
                    entry.instance_count += 1;
                    entry.instance_bytes += block_size as u64;
                    if entry.first_seen_payload == 0 {
                        entry.first_seen_payload = (block as usize) + header_size;
                    }
                }
                offset += block_size;
            }
        }
    }

    let mut snapshots: Vec<TypeSnapshot> = buckets
        .into_iter()
        .map(|(type_desc_addr, bucket)| {
            let td = type_desc_addr as *const TypeDesc;
            // Safety: addresses came from live block tags. The TypeDesc is
            // emitted as read-only data alongside the JIT module that defined
            // it; it stays valid as long as the module is live, which is a
            // necessary precondition for any block tagged with it to exist.
            let (size, vtable_len, has_finalizer) = unsafe {
                ((*td).size, (*td).vtable_len, (*td).finalizer.is_some())
            };
            TypeSnapshot {
                display_name: format_type_name(type_desc_addr),
                type_desc_addr,
                size,
                vtable_len,
                has_finalizer,
                instance_count: bucket.instance_count,
                instance_bytes: bucket.instance_bytes,
            }
        })
        .collect();
    // Stable sort: largest live-bytes first, then by name for determinism.
    snapshots.sort_by(|a, b| {
        b.instance_bytes
            .cmp(&a.instance_bytes)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    snapshots
}

#[derive(Default)]
struct TypeBucket {
    instance_count: u64,
    instance_bytes: u64,
    first_seen_payload: usize,
}

#[derive(Default)]
struct ClusterOccupancy {
    live_blocks: u64,
    live_bytes: u64,
    free_blocks: u64,
    free_bytes: u64,
    largest_free_block: u64,
}

fn walk_cluster_occupancy(cluster: &Cluster) -> ClusterOccupancy {
    let mut o = ClusterOccupancy::default();
    let mut offset = 0usize;
    unsafe {
        while offset < cluster.bump {
            let hdr = cluster.base.add(offset) as *const BlockHeader;
            let block_size = (*hdr).block_size;
            if block_size < 32 || offset + block_size > cluster.bump {
                break;
            }
            let type_bits = (*hdr).tag & !1usize;
            if type_bits == 0 {
                o.free_blocks += 1;
                o.free_bytes += block_size as u64;
                if (block_size as u64) > o.largest_free_block {
                    o.largest_free_block = block_size as u64;
                }
            } else {
                o.live_blocks += 1;
                o.live_bytes += block_size as u64;
            }
            offset += block_size;
        }
    }
    o
}

/// Resolve a `TypeDesc` address to its qualified type name (e.g.
/// `"Stores.StoreDesc"`). Falls back to `Type@0xADDR` for fabricated
/// or hand-rolled TypeDescs that codegen didn't emit a name for —
/// chiefly the test helpers; production-emitted TypeDescs always
/// carry a name pointer.
fn format_type_name(type_desc_addr: usize) -> String {
    crate::kernel_sys::type_desc_qualified_name_string(type_desc_addr as i64)
        .unwrap_or_else(|| format!("Type@0x{type_desc_addr:x}"))
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Rendering
// ─────────────────────────────────────────────────────────────────────────────

/// Mode selector for `render`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Counters,
    Clusters,
    Roots,
    Types,
    Full,
}

impl HeapSnapshot {
    pub fn render(&self, mode: RenderMode) -> String {
        let mut out = String::new();
        out.push_str("newcp heap snapshot\n");
        match mode {
            RenderMode::Counters => self.push_counters(&mut out),
            RenderMode::Clusters => {
                self.push_counters(&mut out);
                self.push_clusters(&mut out);
            }
            RenderMode::Roots => {
                self.push_counters(&mut out);
                self.push_modules(&mut out);
            }
            RenderMode::Types => {
                self.push_counters(&mut out);
                self.push_types(&mut out);
            }
            RenderMode::Full => {
                self.push_counters(&mut out);
                self.push_clusters(&mut out);
                self.push_types(&mut out);
                self.push_modules(&mut out);
            }
        }
        out
    }

    fn push_counters(&self, out: &mut String) {
        let c = &self.counters;
        out.push_str("\ncounters:\n");
        out.push_str(&format!(
            "  alloc-lifetime:        {:>10} blocks   {:>14} bytes\n",
            c.alloc_blocks_lifetime, c.alloc_bytes_lifetime
        ));
        out.push_str(&format!(
            "  free-lifetime:         {:>10} blocks   {:>14} bytes\n",
            c.free_blocks_lifetime, c.free_bytes_lifetime
        ));
        out.push_str(&format!(
            "  live (post-sweep):     {:>10} blocks   {:>14} bytes\n",
            c.live_blocks, c.live_bytes
        ));
        out.push_str(&format!(
            "  peak-live:                                  {:>14} bytes\n",
            c.peak_live_bytes
        ));
        out.push_str(&format!(
            "  cluster-count: {}   module-roots: {}\n",
            c.cluster_count, c.module_root_count
        ));
        out.push_str(&format!(
            "  alloc-paths: bump {} / free-list {} / grow-events {}\n",
            c.bump_path_blocks, c.free_list_path_blocks, c.grow_events
        ));
        out.push_str(&format!(
            "  collect-cycles: {} (last {:.3} ms, total {:.3} ms, last reclaimed {} bytes)\n",
            c.collect_cycles,
            c.collect_last_nanos as f64 / 1_000_000.0,
            c.collect_total_nanos as f64 / 1_000_000.0,
            c.collect_last_reclaimed_bytes,
        ));
    }

    fn push_clusters(&self, out: &mut String) {
        out.push_str("\nclusters:\n");
        if self.clusters.is_empty() {
            out.push_str("  <none>\n");
            return;
        }
        for c in &self.clusters {
            out.push_str(&format!(
                "  #{}  base 0x{:x}  size {} B  bump {} B\n",
                c.index, c.base, c.size, c.bump
            ));
            out.push_str(&format!(
                "      live {} blocks ({} B)  free {} blocks ({} B, largest {} B, frag {:.2})\n",
                c.live_blocks,
                c.live_bytes,
                c.free_blocks,
                c.free_bytes,
                c.largest_free_block,
                c.fragmentation_ratio,
            ));
        }
    }

    fn push_modules(&self, out: &mut String) {
        out.push_str("\nmodule roots:\n");
        if self.modules.is_empty() {
            out.push_str("  <none>\n");
            return;
        }
        for m in &self.modules {
            out.push_str(&format!(
                "  {:<16} var_base 0x{:x}  offsets {:?}\n",
                m.name, m.var_base, m.offsets
            ));
            for (offset, value) in m.offsets.iter().zip(m.current_pointer_values.iter()) {
                out.push_str(&format!("      [+{offset}]  -> 0x{value:x}\n"));
            }
        }
    }

    fn push_types(&self, out: &mut String) {
        out.push_str("\ntypes (sorted by live bytes desc):\n");
        if self.types.is_empty() {
            out.push_str("  <none>\n");
            return;
        }
        for t in &self.types {
            out.push_str(&format!(
                "  {:<24} instances {:>6}  bytes {:>10}  size {} B  vtable {} slots  finalizer {}\n",
                t.display_name,
                t.instance_count,
                t.instance_bytes,
                t.size,
                t.vtable_len,
                if t.has_finalizer { "YES" } else { "no" },
            ));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// JSON output (lightweight, no serde dependency)
// ─────────────────────────────────────────────────────────────────────────────

impl HeapSnapshot {
    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(1024);
        s.push('{');
        s.push_str(&format!(
            "\"taken_at_ns\":{},",
            self.taken_at_ns_since_epoch
        ));
        s.push_str("\"counters\":");
        json_counters(&self.counters, &mut s);
        s.push_str(",\"clusters\":[");
        for (i, c) in self.clusters.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            json_cluster(c, &mut s);
        }
        s.push_str("],\"modules\":[");
        for (i, m) in self.modules.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            json_module(m, &mut s);
        }
        s.push_str("],\"types\":[");
        for (i, t) in self.types.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            json_type(t, &mut s);
        }
        s.push_str("]}");
        s
    }
}

fn json_counters(c: &HeapCountersSnapshot, s: &mut String) {
    s.push_str(&format!(
        "{{\"alloc_blocks_lifetime\":{},\"alloc_bytes_lifetime\":{},\
\"free_blocks_lifetime\":{},\"free_bytes_lifetime\":{},\
\"bump_path_blocks\":{},\"free_list_path_blocks\":{},\"grow_events\":{},\
\"collect_cycles\":{},\"collect_total_nanos\":{},\"collect_last_nanos\":{},\
\"collect_last_reclaimed_bytes\":{},\"live_blocks\":{},\"live_bytes\":{},\
\"cluster_count\":{},\"module_root_count\":{},\"peak_live_bytes\":{}}}",
        c.alloc_blocks_lifetime,
        c.alloc_bytes_lifetime,
        c.free_blocks_lifetime,
        c.free_bytes_lifetime,
        c.bump_path_blocks,
        c.free_list_path_blocks,
        c.grow_events,
        c.collect_cycles,
        c.collect_total_nanos,
        c.collect_last_nanos,
        c.collect_last_reclaimed_bytes,
        c.live_blocks,
        c.live_bytes,
        c.cluster_count,
        c.module_root_count,
        c.peak_live_bytes,
    ));
}

fn json_cluster(c: &ClusterSnapshot, s: &mut String) {
    s.push_str(&format!(
        "{{\"index\":{},\"base\":{},\"size\":{},\"bump\":{},\
\"live_blocks\":{},\"live_bytes\":{},\"free_blocks\":{},\"free_bytes\":{},\
\"largest_free_block\":{},\"fragmentation_ratio\":{}}}",
        c.index,
        c.base,
        c.size,
        c.bump,
        c.live_blocks,
        c.live_bytes,
        c.free_blocks,
        c.free_bytes,
        c.largest_free_block,
        c.fragmentation_ratio,
    ));
}

fn json_module(m: &ModuleRootSnapshot, s: &mut String) {
    s.push_str(&format!(
        "{{\"name\":\"{}\",\"var_base\":{},\"offsets\":{:?},\"current_pointer_values\":{:?}}}",
        m.name.replace('"', "\\\""),
        m.var_base,
        m.offsets,
        m.current_pointer_values,
    ));
}

fn json_type(t: &TypeSnapshot, s: &mut String) {
    s.push_str(&format!(
        "{{\"display_name\":\"{}\",\"type_desc_addr\":{},\"size\":{},\
\"vtable_len\":{},\"has_finalizer\":{},\"instance_count\":{},\"instance_bytes\":{}}}",
        t.display_name.replace('"', "\\\""),
        t.type_desc_addr,
        t.size,
        t.vtable_len,
        t.has_finalizer,
        t.instance_count,
        t.instance_bytes,
    ));
}

// `ModuleRoots` is referenced in the imports above but only needed at type-
// resolution time inside `gc.rs`; suppress the unused-import lint here.
#[allow(dead_code)]
type _UseModuleRoots = ModuleRoots;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gc::{
        __newcp_init_gc, __newcp_new_rec, __newcp_register_module_named, collect, Finalizer,
        TypeDesc,
    };
    fn lock_tests() -> std::sync::MutexGuard<'static, ()> {
        crate::gc::lock_tests_global()
    }

    /// Wipe enough state that the introspection tests start clean. Cannot
    /// share `reset_gc` from the gc module's tests (private), so we replicate
    /// the necessary teardown via the public-ish surface.
    fn reset_for_test() {
        // Reset counters; clusters/modules cleared via with_locked_state.
        reset_counters();
        gc::with_locked_state(|_| ());
        crate::gc::reset_for_test();
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

    #[test]
    fn counters_track_alloc_path_attribution() {
        let _t = lock_tests();
        reset_for_test();

        let stack_marker = [0usize; 64];
        let base = unsafe { stack_marker.as_ptr().add(64) as usize };
        unsafe { __newcp_init_gc(base as *const u8) };

        let td = make_leaf_typedesc(64);
        for _ in 0..10 {
            let _ = unsafe { __newcp_new_rec(td) };
        }
        let c = current_counters();
        assert_eq!(c.alloc_blocks_lifetime, 10);
        assert!(c.alloc_bytes_lifetime > 0);
        // First allocations come from the bump path of a freshly-grown cluster.
        assert!(c.bump_path_blocks > 0);
        assert_eq!(c.free_list_path_blocks, 0);
        assert!(c.cluster_count >= 1);
        assert!(c.grow_events >= 1);
    }

    #[test]
    fn counters_track_collect_and_reclaim() {
        let _t = lock_tests();
        reset_for_test();

        // No __newcp_init_gc → no stack scan → deterministic reclamation.
        // Note: the alloc path runs an opportunistic collect on heap-grow,
        // so collect_cycles is not necessarily 0 after the first alloc.
        // Measure the delta around the *explicit* collect() call instead.
        let td = make_leaf_typedesc(64);
        for _ in 0..50 {
            let _ = unsafe { __newcp_new_rec(td) };
        }
        let before = current_counters();
        let cycles_before = before.collect_cycles;

        collect();

        let after = current_counters();
        assert_eq!(
            after.collect_cycles,
            cycles_before + 1,
            "collect() must record one cycle"
        );
        assert!(after.free_blocks_lifetime >= 50);
        assert!(after.collect_last_reclaimed_bytes > 0);
        // No stack scan + no roots = nothing live.
        assert_eq!(after.live_blocks, 0);
        assert_eq!(after.live_bytes, 0);
    }

    #[test]
    fn snapshot_summarises_cluster_and_module_roots() {
        let _t = lock_tests();
        reset_for_test();

        // Register a named module and an allocation.
        let slot: Box<*mut u8> = Box::new(std::ptr::null_mut());
        let slot_ptr = Box::into_raw(slot);
        let offsets = [0isize];
        let name = "TestModule";
        unsafe {
            __newcp_register_module_named(
                name.as_ptr(),
                name.len(),
                slot_ptr as *const u8,
                offsets.as_ptr(),
                offsets.len(),
            );
        }

        let td = make_leaf_typedesc(64);
        let payload = unsafe { __newcp_new_rec(td) };
        unsafe { slot_ptr.write(payload) };

        let snap = take_snapshot();
        assert!(!snap.clusters.is_empty());
        let cluster = &snap.clusters[0];
        assert!(cluster.live_blocks >= 1);
        assert!(cluster.live_bytes >= 64);

        let module = snap
            .modules
            .iter()
            .find(|m| m.name == "TestModule")
            .expect("named module appears in snapshot");
        assert_eq!(module.offsets, vec![0]);
        assert_eq!(module.current_pointer_values, vec![payload as usize]);

        // Type catalog should account for our allocation.
        let leaf = snap
            .types
            .iter()
            .find(|t| t.type_desc_addr == td as usize)
            .expect("type catalog has our typedesc");
        assert_eq!(leaf.instance_count, 1);
        assert!(leaf.instance_bytes >= 64);

        unsafe { drop(Box::from_raw(slot_ptr)) };
    }

    #[test]
    fn render_counters_includes_key_fields() {
        let snap = HeapSnapshot {
            counters: HeapCountersSnapshot {
                alloc_blocks_lifetime: 17,
                alloc_bytes_lifetime: 4096,
                collect_cycles: 3,
                ..Default::default()
            },
            ..Default::default()
        };
        let text = snap.render(RenderMode::Counters);
        assert!(text.contains("alloc-lifetime:"));
        assert!(text.contains("17 blocks"));
        assert!(text.contains("collect-cycles: 3"));
    }

    #[test]
    fn json_round_trip_is_well_formed() {
        let snap = HeapSnapshot {
            counters: HeapCountersSnapshot {
                alloc_blocks_lifetime: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        let json = snap.to_json();
        // Smoke check: expected keys and balanced braces.
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("\"alloc_blocks_lifetime\":1"));
        assert!(json.contains("\"clusters\":[]"));
    }
}
