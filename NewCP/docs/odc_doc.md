# `.odc` Documents in Memory — Pointer Width and Arena Question

## Context

This note answers two related questions about how `.odc` (and YAML-projected) documents should live in memory once NewCP is loading them as runtime objects:

1. Are 32-bit pointers inside documents OK in the 64-bit world?
2. Do documents need their own memory arena?

Sources surveyed:

- [`NewCP/src/newcp-odc/`](../src/newcp-odc/) — the round-trip reader / writer, especially [`envelope.rs`](../src/newcp-odc/src/envelope.rs), [`text_model.rs`](../src/newcp-odc/src/text_model.rs), [`lifted.rs`](../src/newcp-odc/src/lifted.rs).
- [`docs/odc_yaml_format.md`](odc_yaml_format.md) — the on-wire format and YAML projection.
- [`docs/blackbox-jit-compatibility.md`](blackbox-jit-compatibility.md) — runtime ABI policy, especially §3a (ABI sizing).
- [`docs/garbage-collection.md`](garbage-collection.md) — the unified mark-and-sweep GC design.
- Legacy CP source under `review-md/`: `System/Mod/Stores.odc.md`, `Text/Mod/Models.odc.md`.

## Two distinct in-memory shapes

It is important to separate these — the answers differ.

### 1. The reader's parse AST (what `newcp-odc` produces today)

```rust
Document { source_path, size, root: StoreNode, bytes: Vec<u8> }
```

defined at [`envelope.rs:34`](../src/newcp-odc/src/envelope.rs). It owns the original file bytes and a tree of `StoreNode`s reached via `children: Vec<StoreNode>`. View bodies (`TextModelBody`, `StdViewBody`, `LiftedPiece`, …) decode on demand from those bytes.

This is a **read / round-trip representation**, not the live document. It is purely Rust-owned, has no CP-runtime visibility, and never enters the GC.

### 2. The live BlackBox document graph (what NewCP will materialise when the document is "open")

From the legacy `Stores` and `TextModels` modules:

- `Stores.Store = POINTER TO ABSTRACT RECORD { dlink: Domain; era, id: INTEGER; isElem: BOOLEAN }` — every store node is a GC-managed pointer.
- `Stores.Domain` — the per-document logical container. It owns a `StoreDict` (id → store) and a `TrapCleaner`. **It is not a separate allocator.** It is just the GC root that keeps the document's stores reachable.
- `TextModels.StdModel`: `len`, `id`, `era`, `trailer: Run` (a circular doubly-linked piece list), `pc: PieceCache`, `attrs: AttrDict`, optional `SpillFile` for very large text.
- `Run = POINTER TO RECORD { prev, next: Run; len: INTEGER }` with subtypes `Piece` (short char run), `LPiece` (long char run), `ViewRef` (embedded view).

A live document is therefore a **graph of `POINTER TO RECORD` instances on the unified GC heap**, rooted at `Documents.StdDocument`, scoped logically by `Domain`.

## Are 32-bit pointers inside documents OK?

There are no 32-bit pointers in BlackBox documents — neither on the wire nor in memory. The question is a category confusion worth unpicking:

| Value | Wire width | In-memory (legacy) | In-memory (NewCP) |
|---|---|---|---|
| `Next`, `Down`, `Length` (file offsets) | i32 (`INTEGER`) | n/a (file-only) | u64 in `StoreNode.body_pos` / `body_len` (already widened) |
| `ObjectId`, `TypeId` (dictionary indices) | i32 | i32 (`INTEGER`) | keep i32 — they are indices, not addresses |
| `id`, `era`, `len`, `offset`, `pos`, `beg`, `end` | i32 | i32 (`INTEGER`) | keep i32 — semantic counters and positions |
| `Run.prev`, `Run.next`, `StdModel.trailer`, `Store.dlink` | n/a | host pointer (32 on x86, 64 on x64) | **must be 64-bit** |
| `attr` pointers, `view: Views.View`, `text: Files.File`, … | n/a | host pointer | **must be 64-bit** |

The wire format's 32-bit `INTEGER` is fine forever. A single store body is bounded by `len: i32` ≈ 2 GB, and real BlackBox documents are KB–MB; the corpus we've already round-tripped (675 / 675 files in the BlackBox 1.7 tree) tops out at 207 KB. There is no realistic scenario where a single `.odc` body needs more than 31 bits of length.

The 32-bit `INTEGER` fields that survive into memory (`id`, `era`, attribute-pool indexes, piece lengths, dictionary IDs) are **counters and indices, not addresses**. Keep them at their natural width — using i64 for an attribute-pool index would just waste cache lines without buying anything.

The places where width matters are the **pointers between records** (`Run.next`, `Store.dlink`, attribute pointers, embedded view pointers, `SpillFile.file`). Those are full host pointers and must be 64-bit on NewCP. This matches [blackbox-jit-compatibility.md §3a](blackbox-jit-compatibility.md) directly:

> Any field that names an address-like location must be treated as 64-bit unless there is a specific bounded reason not to. Byte offsets, table indexes, and compact encoded metadata may remain narrower, but only when they are deliberately specified as non-address values.

## Should we use a per-document arena?

**No — and BlackBox didn't either.** Three reasons.

### 1. Documents are open-ended graphs, not closed regions

- **Cross-document references exist.** `StdLinks.Link` carries a command string like `"StdCmds.OpenDoc('../../Docu/BB-License.odc')"`; opening that traverses to *another* document's graph. Even within one document, `StdReader` and `StdWriter` records hold pointers into a model from outside. An arena assumes a closed pointer set; documents don't have one.
- **Models are shared.** Multiple views can present the same `TextModels.StdModel` (split panes, find-results popups). Lifetime is "the GC says so", not "the document is closed".
- **Editing churns allocations.** Every keystroke can split a `Run` and allocate a new `Piece`. A bump arena doesn't reclaim within its lifetime, so an interactive editor would blow it up.
- **Reflection holds references.** `Meta.Item` enumeration, `Documents.Controller`, `Properties.Property` — many subsystems can latch a pointer into document state and outlive a "close".

### 2. The GC is already designed for this

[garbage-collection.md](garbage-collection.md) specifies:

- conservative stack scanning, precise heap tracing via `TypeDesc.ptroffs`;
- module roots via `varBase + ptrs` (§3);
- a cluster-based allocator with per-block headers carrying mark bits.

A `Documents.StdDocument` is exactly the kind of closed-ish subgraph the mark phase reclaims for free when no roots reach it any more. Adding an arena alongside means either (a) the arena is opaque to the GC — and any pointer that escapes it dangles — or (b) the GC has to scan the arena, which buys nothing over the unified heap.

### 3. BlackBox's actual answer to memory pressure was `SpillFile`, not arenas

`StdModel` carries an optional `SpillFile`: huge text spills into a temp file rather than staying resident. That is the precedent if NewCP ever needs to handle multi-MB documents — file-backed overflow, not arena allocation.

## Concrete recommendations for the live-document path

When the load step (newcp-odc AST → live CP graph) is implemented, hold this line:

1. **Allocate document records on the standard GC heap.** Each `Stores.Store` becomes a `__newcp_new_rec(typedesc)` block; pointer offsets in `TypeDesc.ptroffs` describe the inter-store pointers so the tracer follows them.
2. **Use `Domain` as a logical scope only.** It is a single GC-rooted record holding the root `Store` plus a cleaner. Keep it — it gives "close document" semantics for free (drop the root → the next collection sweeps it).
3. **Keep wire-format integers narrow on purpose.** `id`, `era`, attribute indexes, piece lengths — i32 is correct, matches CP `INTEGER`, and is what every existing BB algorithm assumes. Do not widen them "to future-proof" — that would just diverge from the spec.
4. **Widen anything that names an address.** All `Run.prev` / `next`, `Store.dlink`, `Attributes` pointers, embedded `Views.View` pointers, `Files.File` handles. This follows the ABI sizing policy.
5. **Don't repurpose tag bits for offsets.** [garbage-collection.md §1.2](garbage-collection.md) reserves the LSBs of the `Tag` pointer for the GC mark bit. Any "pack offset into pointer" trick (compressed-oops style) would conflict with that.
6. **Discard the source bytes once internalisation finishes.** The reader's `bytes: Vec<u8>` is part of the round-trip path only. The live document owns the decoded graph; release the byte buffer the same way BlackBox releases its `Files.Reader` after `Stores.ReadStore`.

## Short answer

Documents are normal, GC-managed object graphs with full-width pointers between records. The 32-bit values in the format are non-address indices and positions that happily stay 32-bit. No arena is needed, and adding one would fight both the BlackBox object model and the GC design that is already specified.
