//! Native `StoresSys` module — flat C-ABI shims that re-expose
//! `newcp-odc`'s read-only envelope walker through CP-callable
//! procedures.
//!
//! This is Stores Stage S1 (per docs/stores_module_design.md §11):
//! a CP program can open a `.odc` file, walk its store tree, and
//! read each store's type name + body length. No `Stores.Store`
//! records are instantiated yet — that's S2. The point of S1 is
//! to validate the FFI shape end-to-end before committing to the
//! typed graph.
//!
//! ABI: opaque INTEGER handles, all primitives `extern "C"`. The
//! Stores.cp definition module wraps these in BlackBox-style typed
//! aliases.
//!
//! Handle ABI:
//! - **Document handle**: 1-based index into a process-global
//!   table of open `Document`s. 0 = invalid / not open.
//! - **Store handle**: high 32 bits = document handle, low 32 bits
//!   = node index within that document's flat node list. 0 = NIL.
//!
//! Lifetimes: the document's owned bytes + flattened node list
//! are kept alive by the table entry. Closing the document
//! invalidates all of its store handles; calling a Get* shim with
//! a stale handle returns 0 / empty string.

use std::sync::Mutex;

use newcp_odc::{read_document, Document, StoreKind, StoreNode};

/// Pack a (document, node) pair into a single i64 handle. The
/// document fits in the high 32 bits because we expect at most a
/// few hundred concurrent documents in practice; node indices are
/// bounded by file size (which the wire format caps at 2 GiB per
/// store body, comfortably fitting 32 bits).
const DOC_SHIFT: u32 = 32;
const NODE_MASK: u64 = 0x0000_0000_FFFF_FFFF;

#[inline]
fn pack_store_handle(doc: u32, node: u32) -> i64 {
    (((doc as u64) << DOC_SHIFT) | (node as u64)) as i64
}

#[inline]
fn unpack_store_handle(h: i64) -> Option<(u32, u32)> {
    if h == 0 {
        return None;
    }
    let raw = h as u64;
    let doc = (raw >> DOC_SHIFT) as u32;
    let node = (raw & NODE_MASK) as u32;
    if doc == 0 {
        return None;
    }
    Some((doc, node))
}

/// Flat record per `StoreNode` for handle-based navigation. The
/// hierarchy is reconstructed by indices: each entry knows its
/// first-child (if any) and its next-sibling (if any).
#[derive(Debug, Clone)]
struct FlatNode {
    type_name: String,
    body_len: u64,
    kind: StoreKind,
    first_child: Option<u32>,
    next_sibling: Option<u32>,
}

struct DocumentEntry {
    /// Owned `Document` keeps the byte buffer alive in case S2 ever
    /// wants per-store body access (today only the flat node list
    /// is exposed).
    #[allow(dead_code)]
    doc: Document,
    nodes: Vec<FlatNode>,
}

struct StoresState {
    documents: Vec<Option<DocumentEntry>>,
}

impl StoresState {
    const fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }
}

static STATE: Mutex<StoresState> = Mutex::new(StoresState::new());

/// Walk the StoreNode tree once, producing the flat Vec used for
/// handle navigation. The root becomes index 0; children come
/// after their parents in DFS order. Returns the populated Vec.
fn flatten_tree(root: &StoreNode) -> Vec<FlatNode> {
    let mut out: Vec<FlatNode> = Vec::new();
    flatten_recursive(root, &mut out);
    out
}

fn flatten_recursive(node: &StoreNode, out: &mut Vec<FlatNode>) -> u32 {
    let my_idx = out.len() as u32;
    out.push(FlatNode {
        type_name: node.display_kind(),
        body_len: node.body_len,
        kind: node.kind,
        first_child: None,
        next_sibling: None,
    });
    let mut prev_child: Option<u32> = None;
    for child in &node.children {
        let child_idx = flatten_recursive(child, out);
        match prev_child {
            None => {
                out[my_idx as usize].first_child = Some(child_idx);
            }
            Some(p) => {
                out[p as usize].next_sibling = Some(child_idx);
            }
        }
        prev_child = Some(child_idx);
    }
    my_idx
}

/// Decode a CP open-array `IN path: ARRAY OF CHAR` (UTF-32, zero-
/// terminated, length passed alongside) into a Rust `String`.
fn decode_path(ptr: *const u32, max_len: i64) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let cap = if max_len <= 0 { 4096 } else { max_len as usize };
    let mut s = String::with_capacity(64);
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

/// Write a UTF-32 zero-terminated codepoint stream (capped at
/// `cap - 1`) into `dst`, terminator included.
fn write_utf32_out(s: &str, dst: *mut u32, cap: i64) {
    if dst.is_null() || cap <= 0 {
        return;
    }
    let cap_chars = (cap as usize).saturating_sub(1);
    let mut i = 0usize;
    for c in s.chars() {
        if i >= cap_chars {
            break;
        }
        unsafe { *dst.add(i) = c as u32 };
        i += 1;
    }
    unsafe { *dst.add(i) = 0 };
}

/// `StoresSys.OpenDocument(IN path: ARRAY OF CHAR): INTEGER`.
/// Returns a 1-based document handle, or 0 on failure (file
/// missing, bad magic, parse error).
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_open_document(path_ptr: *const u32, path_len: i64) -> i64 {
    let path = decode_path(path_ptr, path_len);
    if path.is_empty() {
        return 0;
    }
    let Ok(doc) = read_document(&path) else {
        return 0;
    };
    let nodes = flatten_tree(&doc.root);
    let mut state = STATE.lock().expect("stores state mutex poisoned");

    // Reuse a vacated slot if there is one.
    for (i, slot) in state.documents.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(DocumentEntry { doc, nodes });
            return (i + 1) as i64;
        }
    }
    state.documents.push(Some(DocumentEntry { doc, nodes }));
    state.documents.len() as i64
}

/// `StoresSys.CloseDocument(handle: INTEGER)`. Releases the
/// document's owned bytes and node table. Subsequent operations
/// on store handles from this document return 0 / empty.
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_close_document(handle: i64) {
    if handle <= 0 {
        return;
    }
    let mut state = STATE.lock().expect("stores state mutex poisoned");
    let idx = (handle - 1) as usize;
    if let Some(slot) = state.documents.get_mut(idx) {
        *slot = None;
    }
}

/// `StoresSys.RootStore(doc: INTEGER): INTEGER`. Returns a store
/// handle for the root, or 0 if the document is closed / invalid.
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_root_store(doc: i64) -> i64 {
    if doc <= 0 {
        return 0;
    }
    let state = STATE.lock().expect("stores state mutex poisoned");
    let idx = (doc - 1) as usize;
    let Some(Some(entry)) = state.documents.get(idx) else {
        return 0;
    };
    if entry.nodes.is_empty() {
        return 0;
    }
    pack_store_handle(doc as u32, 0)
}

fn with_node<F, R>(handle: i64, f: F) -> Option<R>
where
    F: FnOnce(&FlatNode) -> R,
{
    let (doc, node_idx) = unpack_store_handle(handle)?;
    let state = STATE.lock().expect("stores state mutex poisoned");
    let entry = state.documents.get((doc - 1) as usize)?.as_ref()?;
    let node = entry.nodes.get(node_idx as usize)?;
    Some(f(node))
}

/// `StoresSys.FirstChild(store: INTEGER): INTEGER`. Returns the
/// first child's store handle, or 0 if the store has no children.
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_first_child(store: i64) -> i64 {
    let Some((doc, _)) = unpack_store_handle(store) else {
        return 0;
    };
    with_node(store, |n| n.first_child)
        .flatten()
        .map(|c| pack_store_handle(doc, c))
        .unwrap_or(0)
}

/// `StoresSys.NextSibling(store: INTEGER): INTEGER`. Returns the
/// next sibling's store handle, or 0 if there is no next sibling.
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_next_sibling(store: i64) -> i64 {
    let Some((doc, _)) = unpack_store_handle(store) else {
        return 0;
    };
    with_node(store, |n| n.next_sibling)
        .flatten()
        .map(|s| pack_store_handle(doc, s))
        .unwrap_or(0)
}

/// `StoresSys.GetTypeName(store: INTEGER; OUT name: ARRAY OF CHAR)`.
/// Writes the store's type name (e.g. "TextModels.StdModel") into
/// the OUT array. Empty string for nil / link / newlink stores.
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_get_type_name(store: i64, name: *mut u32, name_len: i64) {
    let s = with_node(store, |n| n.type_name.clone()).unwrap_or_default();
    write_utf32_out(&s, name, name_len);
}

/// `StoresSys.GetBodyLen(store: INTEGER): INTEGER`. Returns the
/// number of body bytes the store carries, 0 for stores without
/// body (nil/link/newlink).
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_get_body_len(store: i64) -> i64 {
    with_node(store, |n| n.body_len as i64).unwrap_or(0)
}

/// `StoresSys.GetKind(store: INTEGER): INTEGER`. Returns the
/// wire-format kind tag:
/// 0 = nil, 1 = link, 2 = newlink, 3 = store, 4 = elem.
/// (Matches the order in which the variants are declared on
/// `StoreKind`; convenient for `CASE` dispatch in CP.)
#[unsafe(no_mangle)]
pub extern "C" fn stores_sys_get_kind(store: i64) -> i64 {
    with_node(store, |n| match n.kind {
        StoreKind::Nil => 0,
        StoreKind::Link => 1,
        StoreKind::NewLink => 2,
        StoreKind::Store => 3,
        StoreKind::Elem => 4,
    })
    .unwrap_or(0)
}

// ─── Native module registration ─────────────────────────────────────────

use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

/// Shared procedure list exposed under both the `StoresSys` and
/// `Stores` module names (analogous to the `KernelSys` / `Kernel`
/// pair). One Rust function backs each pair.
fn stores_exports() -> &'static [(&'static str, *const ())] {
    &[
        ("OpenDocument",  stores_sys_open_document  as *const ()),
        ("CloseDocument", stores_sys_close_document as *const ()),
        ("RootStore",     stores_sys_root_store     as *const ()),
        ("FirstChild",    stores_sys_first_child    as *const ()),
        ("NextSibling",   stores_sys_next_sibling   as *const ()),
        ("GetTypeName",   stores_sys_get_type_name  as *const ()),
        ("GetBodyLen",    stores_sys_get_body_len   as *const ()),
        ("GetKind",       stores_sys_get_kind       as *const ()),
    ]
}

fn build_artifact(module_name: &str, summary: &'static str) -> NativeModuleArtifact {
    let entries = stores_exports();
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

pub fn stores_sys_native_module_artifact() -> NativeModuleArtifact {
    build_artifact(
        "StoresSys",
        "Rust-hosted flat-API .odc envelope walker (Stores S1)",
    )
}

pub fn stores_native_module_artifact() -> NativeModuleArtifact {
    build_artifact(
        "Stores",
        "Rust-hosted typed Stores surface (S1: read-only walker)",
    )
}

/// Backwards-compatible alias for the prior single-name registration.
pub fn native_module_artifact() -> NativeModuleArtifact {
    stores_sys_native_module_artifact()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walk the empty document — `Empty.odc` from the BlackBox 1.7
    /// distribution is conventionally a `Documents.StdDocument`
    /// wrapping a single `TextViews.StdView` whose text is empty.
    /// We just check that opening an arbitrary `.odc` works and
    /// that the root has the expected shape.
    #[test]
    fn open_walk_close_a_real_odc() {
        // Try several candidate paths; this lets the test work
        // from either the project root or the runtime crate dir.
        let candidates = [
            "Mod/Tests/Empty.odc",
            "../../Mod/Tests/Empty.odc",
            "NewCP/Mod/Tests/Empty.odc",
        ];
        let mut path_used = None;
        for cand in &candidates {
            if std::path::Path::new(cand).exists() {
                path_used = Some(*cand);
                break;
            }
        }
        // If we don't have a corpus file in the test env, skip —
        // the unit test is informational; integration tests
        // exercise the path proper via run_function.
        let Some(p) = path_used else {
            eprintln!("[stores_sys] no Empty.odc fixture found, skipping");
            return;
        };

        // Encode path as UTF-32 zero-terminated.
        let utf32: Vec<u32> = p.chars().map(|c| c as u32).chain(std::iter::once(0)).collect();
        let h = stores_sys_open_document(utf32.as_ptr(), utf32.len() as i64);
        assert!(h > 0, "OpenDocument should succeed for {p}");

        let root = stores_sys_root_store(h);
        assert!(root != 0, "root store handle should be non-zero");

        // Walk the root's name into a buffer.
        let mut name_buf = [0u32; 256];
        stores_sys_get_type_name(root, name_buf.as_mut_ptr(), 256);
        let name: String = name_buf
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c).unwrap())
            .collect();
        // BlackBox documents always have Documents.StdDocument at
        // the top of the chain.
        assert!(
            name.contains("Documents") || name.contains("StdDocument"),
            "root type name unexpected: {name:?}"
        );

        let kind = stores_sys_get_kind(root);
        assert!(kind == 3 || kind == 4, "root must be a store or elem");

        let body_len = stores_sys_get_body_len(root);
        assert!(body_len > 0, "root body should be non-empty");

        stores_sys_close_document(h);

        // After close, store-handle reads return 0 / empty.
        let post_close = stores_sys_get_body_len(root);
        assert_eq!(post_close, 0, "handles must invalidate after CloseDocument");
    }

    #[test]
    fn invalid_handles_return_zero() {
        assert_eq!(stores_sys_root_store(0), 0);
        assert_eq!(stores_sys_root_store(99999), 0);
        assert_eq!(stores_sys_first_child(0), 0);
        assert_eq!(stores_sys_get_body_len(0xDEAD_BEEF_DEAD_BEEFu64 as i64), 0);
    }
}
