# NewCP — Next Milestones (planning, 2026-05-10)

Snapshot of where the codebase is at the end of the Kernel-bring-up
sprint, the natural sequence of next moves, and the open decisions
that benefit from a wakeful pass before coding resumes.

## Where we ended

Five focused commits today put the BlackBox-equivalent Kernel surface
substantially in place:

| Commit | What landed |
|---|---|
| `638a5d6` | Kernel: first-slice runtime shims (Time, Beep, type reflection, NewObj) |
| `03fab50` | Kernel: language-thread event loop primitive (Loop, Quit, Event, EventHandler) |
| `02c78dc` | Kernel: trap-cleaner stack — Push/Pop + LIFO invocation on trap |
| `8d9c2bd` | Type names on TypeDesc — Kernel.GetTypeName + heap_introspect display |
| `b76c9f4` | ir: unwrap cross-module type aliases to their underlying form |

Test status at the end of day:
- 188 / 188 integration tests
- 49 / 50 runtime tests (the lone failure
  `bootstrap_report_reflects_resident_kernel_and_init` is
  pre-existing, multi-week stale, and unrelated to anything in this
  sprint — it asserts a specific module-list substring that's drifted)
- 29 / 29 loader tests
- All other crates green

Kernel surface state:

| Bound to Rust | Declared, not yet bound |
|---|---|
| Time, Beep | ThisMod, ThisType |
| TypeOf, BaseOf, SizeOf, LevelOf | GetModName, ModOf |
| NewObj | GetLoaderResult |
| Loop, Quit, Event, EventHandler | |
| PushTrapCleaner, PopTrapCleaner | |
| GetTypeName, GetQualifiedTypeName | |

## The next three increments

### 1. `Kernel.ThisMod` / `Kernel.ThisType` — finish the reflection surface

**Why first:** Closes out the Kernel reflection contract before
Stores starts depending on it. Small, self-contained, can land in
one focused commit. Once this is in, the Kernel definition module
is functionally complete for everything in `stores_module_design.md`
except `GetLoaderResult` (a cosmetic — used only for diagnostic
dispatch in `Stores.ThisType`).

**Shape:**
- `Kernel.ThisMod(IN name: ARRAY OF CHAR): Module` — look up by
  module name in the loader's registry. The loader already keeps a
  `KernelState::modules: Vec<ModuleRecord>` list with names. The
  shim reads that registry and returns an opaque `Module` handle.
- `Kernel.ThisType(m: Module; IN name: ARRAY OF CHAR): Type` — find
  the TypeDesc whose qualified name matches `m.name + "." + name`.
  Two implementation options:
  - **(a)** Walk every TypeDesc the runtime has seen so far via
    `BlockHeader.tag` survey of all clusters. Slow but no extra
    state. The dump-heap walker already does this.
  - **(b)** Maintain a runtime-side `{qualified_name → *const
    TypeDesc}` index, populated lazily on the first heap-walk pass
    and refreshed on each heap-grow event. Faster; requires a small
    cache and an invalidation rule.

  (a) is enough for an MVP — the call rate of `ThisType` in
  practice is "once per stored type per document load", which for
  a 1000-type framework is irrelevant cost.

**Estimated scope:** small. One commit, ~150 lines Rust + 10 lines
new declarations on Kernel.cp + 2 integration tests. No codegen
changes.

### 2. Stores Stage S1 — read-only envelope walker

(Per [`stores_module_design.md`](stores_module_design.md) §11.)

**Why next:** Validates the Stores↔`newcp-odc` FFI shape end-to-end
*before* committing to the typed-store-graph design. Lets CP code
load a real `.odc` file and walk its tree without instantiating any
typed records.

**Shape:**
- `Mod/StoresSys.cp` — DEFINITION MODULE, flat C-ABI shims into
  `newcp-odc`. Mirrors the `HostFileSys` / `HostDateSys` pattern.
- `Mod/Stores.cp` — DEFINITION MODULE, typed surface. For S1 only
  the read-only walker functions; full Reader / Writer / Domain
  surface comes in S2.

S1 surface:
```cp
PROCEDURE OpenDocument*(IN path: ARRAY OF CHAR): INTEGER;
  (* returns opaque document handle, 0 on failure *)
PROCEDURE CloseDocument*(handle: INTEGER);

PROCEDURE RootStore*(handle: INTEGER): INTEGER;
  (* returns opaque store handle, 0 if no root *)
PROCEDURE FirstChild*(store: INTEGER): INTEGER;
PROCEDURE NextSibling*(store: INTEGER): INTEGER;

PROCEDURE GetTypeName*(store: INTEGER; OUT name: ARRAY OF CHAR);
PROCEDURE GetBodyLen*(store: INTEGER): INTEGER;
PROCEDURE GetKind*(store: INTEGER): INTEGER;
  (* one of EvNil, EvLink, EvNewLink, EvStore, EvElem *)
```

**CP-side validation:** load `Mod/Tests/StoresProbe.cp` that opens
a small `.odc` from the BlackBox 1.7 corpus, walks the tree, and
asserts the expected type-name sequence. Cleanest fixture is the
675-file `.odc` corpus that `newcp-odc --check` already round-trips.

**Estimated scope:** medium. Two CP modules (~150 lines each), a
new `newcp-stores` Rust crate or just live inside the runtime as
`stores_sys.rs` (TBD — see decisions below).

### 3. Stores Stage S2 — typed Internalize

This is the headline port. NewCP's `Mod/Stores.cp` regrows the
`Store`, `Reader`, `Writer`, `Domain` types and the abstract
`Internalize` / `Externalize` / `CopyFrom` methods. `NewStore` goes
through `Kernel.NewObj`. Each ported view (TextModels.StdModel,
TextModels.Attributes, …) becomes a CP record with a real
`Internalize` body that reads its bytes via the Reader primitives.

**This is multi-commit work** and will take more than a single
session. The S1 step de-risks it by validating the cross-FFI
shape first.

## Open decisions (overnight thinking material)

### A. Stores crate placement

Two reasonable layouts for the Rust side of Stores:

| Option | Pros | Cons |
|---|---|---|
| **New `newcp-stores` crate** | Cleanest layering. Stores depends on `newcp-odc` + `newcp-runtime`; downstream can depend on `newcp-stores` as a separate concern. | Extra crate, longer build chain. |
| **Live in `newcp-runtime/src/stores_sys.rs`** | One fewer crate. Matches the `host_file_sys.rs` / `kernel_sys.rs` pattern. | Mixes concerns inside `newcp-runtime` — though it's already mixed (host_file_sys, host_date_sys, kernel_sys, igui all coexist there). |

Lean: **stay in `newcp-runtime/src/stores_sys.rs`** for S1. The
existing pattern is well-established and a separate crate adds no
real value when the surface is small. If Stores grows substantial
internal state we can split later — Rust cargo move is cheap.

### B. `TypeDesc.module` wiring

Today every TypeDesc's `module` field is `null`. To wire it
properly we'd need:

1. Codegen emits a `ModuleDesc` per source module (currently we
   only have `ModuleRoots` runtime-side, populated by
   `__newcp_register_module_named`).
2. Each TypeDesc's `module` field links to its owning ModuleDesc.

This is a real piece of work. **Workaround for now**: derive the
module from the qualified name string via split-at-last-dot. Slower
on the lookup path but doesn't need codegen changes. `Kernel.ModOf`
becomes a name-lookup helper rather than a direct field read.

Lean: **defer the codegen-side ModuleDesc emission**; ship `ModOf`
as a name-based lookup using the qualified name we now have on
TypeDesc. Cost: one extra lookup per call, which doesn't matter at
the ThisType call rate.

### C. Local-alias unwrap

Today's `b76c9f4` only unwraps **cross-module** type aliases. Local
aliases (`module: None`) still go through the original
`IrType::Named(name)` path. The same problem can in principle bite
a local `TYPE Buf = ARRAY 64 OF CHAR; VAR x: Buf` declaration, but
typically resolves through a different sema path because local
records / aliases get `named_struct_types` registry entries.

**Stores will use lots of local aliases.** Decision: fix
proactively before S2, or fix-on-encounter?

Lean: **fix-on-encounter**. The cross-module fix is targeted; if
the local case bites in Stores S2 we can extend `map_semantic_type`
to take a local-symbols context (invasive plumb but mechanical).

### D. The stale `bootstrap_report_reflects_resident_kernel_and_init` test

Multi-week pre-existing failure. It asserts a specific
`hosted-modules:` substring in the bootstrap report. The substring
has drifted because we've added native modules (HostFileSys,
HostDateSys, KernelSys, Kernel, etc.) since the test was written.

Quick fix: relax the assertion to "contains Console and
HostMenus", drop the brittle order-and-content check.

Lean: **fix tomorrow as a 5-minute clean-up commit before
starting `ThisMod`/`ThisType`**. Carrying a known-fail test
across the workspace is friction.

### E. Should we fix the loader's stale `Mod/HostMenus`?

Looking at `BootstrapReport::new()`: `host_menus` is registered as a
"placeholder facade until CP HostMenus is available". Now that we
have an event loop in Kernel, the next layer up — `HostMenus.OpenApp`,
the MDI machinery — should start moving from "Rust placeholder" to
"CP module on top of Kernel.Loop". That's its own track of work,
not blocking Stores.

Lean: **leave HostMenus alone** for the Stores port. Different
work-stream, different prerequisites (`iGui.OpenChild`, MDI
plumbing, etc.).

## Risk register

- **Heap-side dangling TypeDesc on hot reload.** Documented in
  `docs/heap_introspection.md` and the loader review. The
  `RetiredImageDropPredicate` hook on `LoaderSession` is in place
  but no real probe registers there. With Stores' S2 typed graph
  about to start populating the heap with real CP records, a hot
  reload will cause exactly the dangling-TypeDesc scenario the
  loader review flagged. **Action:** wire a real heap probe
  (consults the heap_introspect type catalog and refuses to drop
  any image whose name range contains a live block's TypeDesc)
  before Stores S2 ships. Or: declare hot-reload-during-stores-
  in-flight unsupported and document it.

- **`SHORT(LONGINT)` truncation bug in `bug_report_short.md`** —
  documented since before this sprint. Affects `Mod/Integers.cp`.
  Not blocking Stores S1/S2. Worth fixing as a small intermezzo
  commit but not load-bearing.

- **Sema name-collision infinite recursion** documented in
  `bug_report_sema_name_collision.md`. The `HostFonts` module ships
  with renamed local types as a workaround. Stores will likely hit
  the same pattern: `Stores.Reader` declares its own internal
  Reader types alongside the ones it imports. If we hit the same
  hang during the port, fall back to renamed locals as HostFonts
  does, and file follow-up.

## What I'd start with at 9 AM

1. **5-minute warm-up:** fix the stale `bootstrap_report` test
   (decision D above). Workspace goes from "1 failing pre-existing"
   to "all green". Removes friction from every subsequent commit.
2. **Half-day:** ship `ThisMod` / `ThisType` (decision B's
   workaround — name-based ModOf is fine for the first slice). One
   focused commit, kernel reflection surface complete.
3. **Rest of the day:** Stores Stage S1. Two CP definition modules,
   a `stores_sys.rs` shim, a `StoresProbe.cp` integration test that
   walks `Tests/Empty.odc` from the corpus. Validates the
   Stores↔newcp-odc FFI end-to-end.

If S1 lands cleanly that's a great day's work; S2 is then a
multi-day project starting the day after.

## Cross-references

- [`stores_module_design.md`](stores_module_design.md) — full
  design for the Stores port.
- [`odc_doc.md`](odc_doc.md) — in-memory document model decisions
  (no arenas; 64-bit pointers; 32-bit indices).
- [`heap_introspection.md`](heap_introspection.md) — heap probe
  hooks the loader's drop-predicate slots into.
- [`stores_module_design.md` §11](stores_module_design.md) — the
  S1–S6 phasing this plan tracks.
