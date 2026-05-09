//! Process-wide MDI child registry.
//!
//! Maps the opaque `child_id` exposed to CP onto the underlying HWND.
//! All registry operations are synchronised by a single Mutex; child
//! lookup is fast (HashMap) and rare (called once per CP-side
//! operation, not on the render path).

#![cfg(windows)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

use windows::Win32::Foundation::HWND;

/// First child id is 2 — the frame itself reserves id 1 for events
/// it pushes (focus, resize, frame-close). Subsequent ids are
/// monotonic and never reused, even after a child closes.
static NEXT_CHILD_ID: AtomicI64 = AtomicI64::new(2);

static REGISTRY: Mutex<Option<HashMap<i64, isize>>> = Mutex::new(None);

fn with_registry<R>(f: impl FnOnce(&mut HashMap<i64, isize>) -> R) -> R {
    let mut guard = REGISTRY.lock().expect("igui child registry mutex poisoned");
    let map = guard.get_or_insert_with(HashMap::new);
    f(map)
}

pub fn allocate_child_id() -> i64 {
    NEXT_CHILD_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn register(child_id: i64, hwnd: HWND) {
    with_registry(|map| {
        map.insert(child_id, hwnd.0 as isize);
    });
}

pub fn unregister(child_id: i64) {
    with_registry(|map| {
        map.remove(&child_id);
    });
}

pub fn hwnd_of(child_id: i64) -> Option<HWND> {
    with_registry(|map| map.get(&child_id).copied().map(|raw| HWND(raw as *mut _)))
}

/// Reverse lookup. Used by the child WndProc to figure out its own id
/// from its HWND when handling events.
pub fn id_of(hwnd: HWND) -> Option<i64> {
    let raw = hwnd.0 as isize;
    with_registry(|map| {
        map.iter()
            .find_map(|(id, h)| if *h == raw { Some(*id) } else { None })
    })
}

/// Snapshot of all registered children. Used by frame teardown to
/// close everything.
pub fn snapshot() -> Vec<(i64, HWND)> {
    with_registry(|map| {
        map.iter()
            .map(|(id, raw)| (*id, HWND(*raw as *mut _)))
            .collect()
    })
}
