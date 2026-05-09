# NewCP `Stores` Module — Design Outline

This note designs the NewCP port of the legacy `Stores` module, taking into account the round-trip codec already shipping in [`newcp-odc`](../src/newcp-odc/) and the in-memory / pointer-width conclusions in [`odc_doc.md`](odc_doc.md).

Sources surveyed:

- legacy CP source: [`review-md/System/Mod/Stores.odc.md`](../../review-md/System/Mod/Stores.odc.md), [`review-md/Text/Mod/Models.odc.md`](../../review-md/Text/Mod/Models.odc.md);
- existing Rust-side codec: [`newcp-odc/src/envelope.rs`](../src/newcp-odc/src/envelope.rs), [`newcp-odc/src/text_model.rs`](../src/newcp-odc/src/text_model.rs), [`newcp-odc/src/lifted.rs`](../src/newcp-odc/src/lifted.rs);
- runtime contracts: [`blackbox-jit-compatibility.md`](blackbox-jit-compatibility.md), [`garbage-collection.md`](garbage-collection.md);
- file-format spec: [`odc_yaml_format.md`](odc_yaml_format.md).

## What the legacy `Stores` module does

Public surface, grouped by responsibility (line numbers refer to [`Stores.odc.md`](../../review-md/System/Mod/Stores.odc.md)):

| Responsibility | Procedures / types |
|---|---|
| Type identity | `TypeName`, `TypePath`, `ThisType`, `GetThisTypeName`, `SameType`, `SamePath` (~493–600) |
| Store creation | `Store` (abstract), `NewStore` (607), `Internalize`/`Externalize`/`CopyFrom` hooks (369–397) |
| Domain (per-document scope) | `Domain`, `NewDomain`, `DomainOf`, `Join`, `Joined`, `Unattached`, `InitDomain` (79–95, 311–349, 2277–2361) |
| Cloning | `BeginCloning`, `EndCloning`, `CopyOf`, era / `sDict` mark scheme (2159–2261) |
| Reader | `Reader` record, `ConnectTo`, `SetPos`, `Pos`, `ReadStore`, primitive `ReadXxx`, `ReadVersion`, `TurnIntoAlien` (1035–1641) |
| Writer | `Writer` record, `ConnectTo`, `SetPos`, `Pos`, `WriteStore`, primitive `WriteXxx`, `WriteVersion` (1641–2145) |
| Wire envelope | `ReadPath`, `WritePath`, type & store dictionaries, type-id encoding (625–905) |
| Aliens | `Alien`, `AlienComp`, `AlienPiece`, `AlienPart`, `InternalizeAlien`, `ExternalizeAlien` (109–137, 929–1033) |
| Trap cleanup | `TrapCleaner`, `Cleanup` (223–249) |
| Constants | kind bytes (`nil/link/store/elem/newlink`), path bytes (`newBase/newExt/oldType`), `dictLineLen`, alien-cause codes (25–69) |

Two facts worth carrying forward:

1. The wire format is open-ended. Type ids and store ids form a self-organising dictionary the writer chose freely; **two writers can encode the same logical document differently** ([odc_yaml_format.md](odc_yaml_format.md), §"Path encoding has multiple wire-equivalent forms"). The implementation must capture wire encoding per store to round-trip byte-identically.
2. The on-wire integers are all 32-bit `INTEGER`. They name file offsets, ids, comments, lengths — never addresses. This is what permits the in-memory representation to be 64-bit pointer-clean without changing the file format ([odc_doc.md](odc_doc.md)).

## What we already have (don't rebuild)

[`newcp-odc`](../src/newcp-odc/) already reproduces the entire byte-level envelope:

- magic + envelope walk via `down`/`next` offsets ([envelope.rs](../src/newcp-odc/src/envelope.rs));
- type-path encoding with `WirePath::Reference | Extension { extensions, terminator }` capturing the wire encoding per store for byte-identical round-trip;
- structured encoders for `TextModels.StdModel`, `TextModels.Attributes`, `StdLinks.Link/Target`, `StdFolds.Fold`, `TextRulers.StdRuler/StdStyle/Attributes`, `TextViews.StdView`, `Controls.*`;
- `--check` sweep validates 675 / 675 BlackBox 1.7 `.odc` files round-trip byte-identically.

This is the canonical wire codec. The NewCP `Stores` module sits **on top of** it, not next to it. Anything wire-format-related (kind dispatch, path encoding, type/store dictionary on-wire shape) must call into `newcp-odc`; we do not duplicate the codec into a separate crate.

## What changes for NewCP

### 1. Two-layer split: native helpers + CP shell

Mirror the same pattern as [`Console`](../src/newcp-runtime/src/console.rs) and `iGui` — a Rust-hosted facade providing the heavy lifting, with a CP module exposing the public API the rest of the framework uses.

- **Native (`newcp-stores` crate, new)** wraps `newcp-odc` and adds:
  - the dispatch glue between a wire `TypeName` and a runtime `Kernel.Type`;
  - the type / store dictionaries indexed by id;
  - reader and writer primitives over a cursor;
  - alien-body capture (verbatim spans, re-using the same opaque mechanism `newcp-odc` already has for unknown view kinds).
- **CP (`Mod/Stores.cp`, new)** declares the public types (`Store`, `Domain`, `Reader`, `Writer`, `Operation`, `Alien*`), the abstract `Internalize/Externalize/CopyFrom` methods every framework type overrides, `NewDomain/Join/CopyOf/...`, and dispatches the byte primitives to the native module via FFI.

The CP module is the public ABI. It must compile and load before any framework module that imports it (Models, Containers, Views, TextModels, Documents, …).

### 2. Crate / module layout

```
NewCP/src/newcp-stores/        # NEW: native helpers
  Cargo.toml                   # depends on newcp-odc and newcp-runtime
  src/lib.rs                   # public FFI entry points: __newcp_stores_*
  src/cursor.rs                # thin wrapper over newcp-odc::primitives::Cursor
  src/dict.rs                  # TypeDict, StoreDict (Vec-backed, id-indexed)
  src/dispatch.rs              # TypeName <-> Kernel.Type lookup, NewStore
  src/alien.rs                 # alien body capture & write-back
  src/io.rs                    # ReadXxx / WriteXxx primitives over cursor

NewCP/Mod/Stores.cp            # NEW: CP module — Domain, Reader, Writer, ...

NewCP/src/newcp-odc/           # EXISTING: keep as the round-trip codec, unchanged
```

`newcp-odc` stays the file-format ground truth. `newcp-stores` consumes it; `Mod/Stores.cp` consumes `newcp-stores` via a small set of FFI symbols.

### 3. In-memory representation (pointer width)

Per [odc_doc.md](odc_doc.md):

- All `Stores.Store`-derived records are GC-managed via `Kernel.NewObj` → `__newcp_new_rec(typedesc)`. Pointer offsets in `TypeDesc.ptroffs` describe inter-store pointers so the tracer follows them. No arenas.
- Pointer fields (`Store.dlink`, `Domain.dlink`, `Domain.s`, `Run.prev/next`, `Attributes` pointers, embedded views) are full 64-bit host pointers.
- `INTEGER` fields stay i32: `id`, `era`, `comment`, `next`, `down`, `len`, `pos`, `cause`, `level`, `nextElemId`, `nextTypeId`, `nextStoreId`, attribute-pool indexes. These are indices/counts/positions, never addresses.
- The reader's transient `bytes: Vec<u8>` (in `newcp-odc::Document`) is a load-time scratch buffer only — release it after `Internalize` finishes, the same way the legacy reader releases its `Files.Reader`.

### 4. Type & store dictionaries

Replace the legacy linked-list-of-32-entry-arrays (`TypeDict` / `StoreDict` at [Stores.odc.md:153–173](../../review-md/System/Mod/Stores.odc.md)) with `Vec<TypeEntry>` / `Vec<Option<Store>>` indexed by id. Reasons:

- `O(1)` indexed lookup vs the legacy self-organising walk;
- no per-line allocator pressure;
- matches the wire-format semantic exactly — id is just the dict-insertion order;
- `newcp-odc` already does this in the reader's `type_dict: Vec<TypeEntry>` ([envelope.rs:144](../src/newcp-odc/src/envelope.rs)).

Wire-encoding decisions (`Reference(id)` vs `Extension { extensions, terminator }`) stay captured per-store in the AST as `WirePath` so the writer reproduces byte-identical output even when two equivalent encodings exist.

### 5. Reader / Writer

CP record layout (passed `VAR rd: Reader` like the legacy code):

```
Reader = RECORD
  rider-:        Files.Reader;       (* underlying byte stream      *)
  cancelled-:    BOOLEAN;
  readAlien-:    BOOLEAN;
  cause:         INTEGER;
  st:            ReaderState;        (* next + end positions        *)
  noDomain:      BOOLEAN;
  store:         Store;              (* root after ReadStore        *)

  (* Hidden state held by the native side, opaque to CP code. *)
  native:        ANYPTR              (* handle into newcp-stores    *)
END;
```

The native handle owns the cursor, the type dictionary, and the elem/store dictionaries. CP-side `ReadInt`, `ReadXString`, `ReadVersion`, `ReadStore`, etc. dispatch to native FFI:

```
__newcp_stores_reader_open(handle: *Reader, file: *FilesFile) -> i32
__newcp_stores_reader_close(handle: *Reader) -> i32
__newcp_stores_read_byte(handle: *Reader) -> i32      (* OUT via *Reader.cause *)
__newcp_stores_read_int(handle: *Reader) -> i32
__newcp_stores_read_long(handle: *Reader, lo: *i32, hi: *i32)
__newcp_stores_read_xstring(handle: *Reader, dst: *u8, cap: i32) -> i32
__newcp_stores_read_string  (handle: *Reader, dst: *u16, cap: i32) -> i32
__newcp_stores_read_path    (handle: *Reader, path: *TypePath) -> i32
__newcp_stores_read_kind    (handle: *Reader, OUT kind: *u8) -> i32
... etc, one per primitive
```

`Writer` mirrors the same structure.

`Reader.ReadStore` stays in CP because it has to call `NewStore` (allocates a CP record via `Kernel.NewObj`) and dispatch to the abstract `x.Internalize(rd)` method. The wire-level dance underneath (read kind byte, read path, read `comment/next/down/len`, validate, walk down-chain) is the same kind switch the legacy module uses, just with native primitives doing the byte work.

### 6. Domain — keep as-is

Domain stays a CP `POINTER TO LIMITED RECORD` ([Stores.odc.md:79](../../review-md/System/Mod/Stores.odc.md)). The path-compressed `DomainOf` walk ([Stores.odc.md:325](../../review-md/System/Mod/Stores.odc.md)) is correct on a 64-bit GC heap — it just chases pointers.

`NewDomain`, `Join`, `Joined`, `Unattached`, `InitDomain`, `SetSequencer`, `GetSequencer`: ported verbatim to CP.

### 7. Cloning — keep as-is

`BeginCloning / EndCloning / CopyOf` use the era + per-domain `sDict` to mark already-copied stores ([Stores.odc.md:2159–2261](../../review-md/System/Mod/Stores.odc.md)). The algorithm is independent of pointer width and stays as a CP procedure. It depends on:

- `Kernel.NewObj` to allocate the clone (already specced in [garbage-collection.md](garbage-collection.md));
- `Kernel.PushTrapCleaner / PopTrapCleaner` so an aborted clone unwinds the era marks.

The trap-cleaner integration is **a new requirement on `Kernel`**. Until it lands, ship `CopyOf` with a no-op cleaner and document the gap — losing trap recovery is acceptable for early bring-up but must be closed before user-visible editing.

### 8. Aliens — phased

When `ReadStore` meets a type it can't `ThisType` (unknown module, missing class, version mismatch), the legacy module synthesises an `Alien` store that captures the bytes verbatim so the rest of the document still loads and round-trips. This is what makes BlackBox forgiving of partial subsystems.

We get the byte-capture mechanism for free from `newcp-odc`'s "unknown view kind" fallback ([odc_yaml_format.md](odc_yaml_format.md), §"Unknown view kinds round-trip via verbatim splice"). Phasing:

| Phase | Behaviour | Purpose |
|---|---|---|
| **A1** | Unknown type → hard error with type name + file position. | Smallest first slice — fine while the subsystem set is closed. |
| **A2** | Unknown type → CP `Alien` store with verbatim body bytes; `Externalize` writes them back unchanged. | Round-trip with partial framework. Required for any hand-edited document that might contain types we haven't ported yet. |
| **A3** | Version-tolerant aliens (`alienVersion`, `inconsistentVersion`, `inconsistentType` causes), `TurnIntoAlien` mid-Internalize. | Full BB compatibility. Needed before opening user documents from arbitrary BlackBox installs. |

A2 is the practical target — it lifts the 675-file round-trip guarantee from "files newcp-odc fully understands" to "every file in the corpus".

### 9. Sequencer hook

`Domain.SetSequencer / GetSequencer` is how editors register undo sequencing. The CP shell exposes both verbatim. Sequencer implementation is framework code (`Sequencers` module), not Stores' problem.

### 10. ABI sizing — explicit per field

Following [blackbox-jit-compatibility.md §3a](blackbox-jit-compatibility.md):

| Field | Width | Rationale |
|---|---|---|
| `Store.dlink`, `Domain.dlink`, `Domain.s`, `Reader.store` | 64-bit pointer | Address |
| `Reader.rider`, `Writer.rider` | 64-bit pointer | Address (`Files.File` handle) |
| `Reader.native`, `Writer.native` | 64-bit `ANYPTR` | Opaque Rust handle |
| `Store.id`, `Store.era`, `Domain.level`, `Domain.copyera`, `Domain.nextElemId` | i32 | Index / counter |
| `Reader.cause`, `nextTypeId`, `nextElemId`, `nextStoreId` | i32 | Counter |
| `ReaderState.next`, `ReaderState.end`, `WriterState.linkpos` | i64 *file offset* | Widened from legacy i32 — `Files.File` offsets are 64-bit |
| Wire `comment`, `next`, `down`, `len`, `id`, `comment` | i32 on wire, i32 in memory | Format-bound; ~2 GB ceiling per body is fine |
| `LONGINT` payloads in `ReadLong` / `WriteLong` | i64 in memory, two i32 halves on wire | Match legacy split |

The one quiet widening: file offsets (`ReaderState.next`, `ReaderState.end`, `Writer.SetPos` argument) become i64. The wire-format `next`/`down` deltas stay i32 because no single body can exceed the 31-bit `len`, but absolute positions in a multi-GB file would otherwise truncate. This is the same call `newcp-odc` already makes (`StoreNode.body_pos: u64`).

### 11. Build phases

Bring-up plan, each phase ending in a runnable artifact:

1. **S1 — Read-only envelope reader.** Re-export `newcp-odc::read_document` through a CP-callable surface. CP code can iterate the store tree and print type names + bodies as opaque blobs. No `Stores.Store` instances allocated yet. Verifies the FFI shape end-to-end.
2. **S2 — Typed `Internalize`.** Implement `NewStore` via `Kernel.NewObj`, register the abstract `Internalize` dispatch. Wire reader primitives. End state: `Documents.StdDocument` plus any subset of view types we've ported can be loaded into a typed CP graph.
3. **S3 — `Externalize`.** Wire writer primitives. The structured encoders in `newcp-odc` already exist for the known kinds — we call them from the CP `Externalize` overrides via FFI. Add `--check` parity sweep for the new path.
4. **S4 — Domain / cloning / trap cleaner.** Port `CopyOf` + Domain plumbing. Requires `Kernel.PushTrapCleaner / PopTrapCleaner`.
5. **S5 — Aliens (A2).** Add verbatim-body capture & write-back. End state: full round-trip across every file in the BlackBox 1.7 corpus from the typed CP graph, not just from the `newcp-odc` AST.
6. **S6 — Aliens (A3).** Version-tolerant `TurnIntoAlien` mid-Internalize, alien-cause reporting via `Dialog.ShowParamMsg`.

### 12. Verification

At each phase that touches the wire format, re-run the existing 675-file `odc-yaml --check` sweep over the BlackBox 1.7 tree. The success criterion stays the same as Stage A/B today: every file reads, the typed graph round-trips, the bytes match.

Add a CP-side test that loads a small `.odc` (e.g. `Empty.odc` for S2, `Tour.odc` for S5) into the typed graph and walks it. The test passes if the walked text + attributes match the YAML projection produced by `newcp-odc::document_to_yaml` on the same file.

## Summary

The NewCP `Stores` module is a **thin CP shell over `newcp-odc`**, with a small `newcp-stores` crate providing the cursor / dictionary / dispatch helpers and the FFI surface. The on-wire format is unchanged; the in-memory representation uses 64-bit pointers and unified-GC allocation; aliens come back via the verbatim-byte mechanism `newcp-odc` already has. No arenas, no narrowed pointers, no parallel codec — the existing codec stays the ground truth and the CP shell layers the framework-visible types and abstract methods on top.
