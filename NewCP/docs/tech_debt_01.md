# Tech Debt Remediation Plan — 01

Initial review pass following the import-cache, bounds-check, and method-dispatch
fixes. Each item below has been verified against the current source tree; the
"Evidence" column cites the file and symbol that motivated the entry.

| # | Area | Severity | Effort |
|---|------|----------|--------|
| 1 | Diagnostic propagation through `lower` / `llvm` phases | High | M |
| 2 | Cyclic type reference protection in cross-module flattening | Medium | S |
| 3 | `OptLevel` field is dead config — wire up an LLVM pass pipeline | Medium | M |
| 4 | Precise GC root metadata for moving / generational GC | Low (future) | L |
| 5 | AST / Sema arena allocation to remove `Arc`/`clone` churn | Low | L |
| 6 | `unwrap()` / `expect()` audit in IR lowering | Low | S |

---

## 1. Diagnostic propagation through `lower` and `llvm`

### Status update
Started on 2026-05-05.

Implemented in code:
- `newcp-ir` now carries procedure-scoped lowering diagnostics.
- Known lowering invariant/fallback paths now record diagnostics instead of
  only silently returning a sentinel or trap.
- `lower_from_path(...)` now fails fast with a formatted lowering-diagnostics
  report, so `newcp-llvm` does not proceed on IR that already carries lowering
  errors.
- `dump-ir` and `dump-cfg` now print lowering diagnostics inline with the dump.
- `newcp-llvm` now carries backend diagnostics for the existing non-fatal
  degradation paths that were already present when `strict_unsupported = false`.
- `dump-llvm` and `dump-asm` now print those backend diagnostics inline as
  warnings before the emitted artifact.

Still pending:
- Unify the diagnostic type across sema / IR / LLVM instead of the current
  stage-local lowering/backend diagnostic structs.
- Extend backend aggregation beyond the current soft-degradation paths; most
  `newcp-llvm` failures still return the first `CodegenError` immediately.

### Evidence
- [src/newcp-sema/src/lib.rs](src/newcp-sema/src/lib.rs#L330) defines
  `SemanticDiagnostic` and propagates it through `SemanticModule.diagnostics` /
  `SemanticProcedure.diagnostics`. The infrastructure exists.
- [src/newcp-ir/src/lower.rs](src/newcp-ir/src/lower.rs#L2118) and surrounding
  lowering helpers return `Option<T>` and short-circuit with `?`. There is no
  channel back to the user when lowering silently drops a construct.
- `newcp-llvm` returns `Result<_, CodegenError>` but only the *first* error
  surfaces; the compiler stops instead of collecting.

### Status of original claim
Partially accurate. Sema diagnostics do exist; the missing piece is downstream
phases participating in the same channel.

### Plan
1. Keep the current `newcp-ir` lowering-diagnostic channel as the first downstream stage.
2. Decide whether to reuse `SemanticDiagnostic` directly or introduce a shared
  `newcp-diagnostics` crate before extending the same pattern into LLVM.
3. Extend the current `newcp-llvm` backend diagnostics beyond soft-degradation
  sites so more backend failures can be accumulated instead of only returning
  the first `CodegenError`.
4. Driver (`newcp-driver/src/main.rs`) should eventually gather sema + lowering
  + llvm diagnostics through one formatter and keep exit code `1` on any
  error-severity entry.

### Definition of done
- `newcp-ir` lowering diagnostics remain surfaced in `lower_from_path`,
  `dump-ir`, and `dump-cfg`.
- A new test can feed a module with two independent lowering errors and assert
  both are reported in one pass.
- `newcp-llvm` grows broader aggregation behavior instead of only surfacing the
  first backend error outside today's soft-degradation sites.

---

## 2. Cyclic type reference protection

### Status update
Started on 2026-05-05.

Implemented in code:
- `newcp-ir` now threads a visited set through named-type flattening so
  `collect_named_types(...)` stops on repeated `(module, type)` resolution
  instead of recursing indefinitely.
- Added a direct `newcp-ir` unit test that constructs a two-module imported
  named-type cycle in memory and asserts the cyclic imported type is skipped.

Still pending:
- Add an integration-style regression that exercises the same guard through a
  full driver path if we decide this case should be representable from source.

### Evidence
- [src/newcp-ir/src/lower.rs](src/newcp-ir/src/lower.rs#L2398)
  `flatten_fields_deep_cross_module` recurses across modules through
  `load_cached_import` and again through nested record fields.
- `load_cached_import` itself does *not* recurse — sema treats foreign symbols
  as `NamedTypeKind::Imported` and stops. So the originally suggested
  "import cycle stack overflow" is **not** the real failure mode.
- The real risk is a **cyclic type reference** (e.g. record `A` containing a
  pointer to record `B`, where `B` contains a pointer back to `A` across
  modules). The flattening recursion has no visited-set.

### Status of original claim
Inaccurate as originally framed. Reframed here against the actual recursion
site.

### Plan
1. Keep the current visited-set guard in the named-type flattening path.
2. Decide whether a revisit should stay as "skip this cyclic expansion" or be
  upgraded later into a lowering diagnostic once downstream diagnostics are
  threaded through IR lowering.
3. If source-level coverage is still wanted, add a driver/integration test only
  after confirming the parser and sema permit a reproducible source example.

### Definition of done
- The `newcp-ir` unit test for cross-module named-type cycles stays green.
- No regressions in the touched crate test suite.

---

## 3. Wire `OptLevel` into an LLVM pass pipeline (mem2reg etc.)

### Status update
Started on 2026-05-04.

Implemented in code:
- `CodegenOptions::opt_level` now flows through driver parsing via
  `--opt {none,less,default,aggressive}`.
- `newcp-llvm` now runs LLVM's new-pass-manager pipelines through
  `Module::run_passes(...)` instead of keeping `opt_level` as dead config.
- JIT and assembly generation now use the selected optimization level when
  constructing the target machine / execution engine.

Still pending:
- Add regression tests that prove the optimized LLVM dump actually reduces
  `alloca` noise for representative modules.
- Decide whether `dump-llvm` test coverage should assert exact `alloca` counts
  or just monotonic reduction under `--opt less`.

### Evidence
- [src/newcp-llvm/src/options.rs](src/newcp-llvm/src/options.rs#L10) declares
  `opt_level: OptLevel` and now defaults to `OptLevel::Default` (`O2`).
- A workspace-wide grep for `opt_level` / `OptLevel::` returns three hits, all
  inside `options.rs`. The field is **declared but never read**.
- Generated `dump-llvm` output is unoptimised: every local goes through
  `alloca`/`store`/`load`, which makes string-containment tests fragile and
  hides codegen regressions behind noise.

### Status of original claim
Accurate.

### Plan
1. Keep the current `Module::run_passes("default<O1|O2|O3>", ...)` pipeline as
  the implementation baseline; do not reintroduce the deprecated legacy
  `PassManager` API.
2. Add focused tests for `dump-llvm --opt less` on one or two modules with
  trivially promotable locals.
3. Keep existing golden-ish IR tests on `OptLevel::None` so they remain stable.
4. Decide later whether higher levels should continue to map to LLVM defaults
  (`default<O2>`, `default<O3>`) or move to a custom pass string once there is
  a concrete need.

### Definition of done
- The current `opt_level` wiring remains compile-clean across driver and LLVM
  crates.
- `dump-llvm --opt less` is covered by at least one regression test proving a
  meaningful reduction in promotable stack slots.
- `cargo test -p newcp-tests` is green at both `--opt none` and `--opt less`.

---

## 4. Precise GC root metadata (future / informational)

### Evidence
- [src/newcp-runtime/src/gc.rs](src/newcp-runtime/src/gc.rs#L5) explicitly
  documents **conservative stack scanning** — "no precise stack maps required
  from LLVM."
- [docs/garbage-collection.md](docs/garbage-collection.md) and the
  `Multi-thread roadmap` section in
  [src/newcp-runtime/src/gc.rs](src/newcp-runtime/src/gc.rs#L554) already
  flag precise statepoints as deferred work.

### Status of original claim
Accurate but premature. The conservative scanner is a deliberate MVP choice;
this entry is tracked here only so the dependency is visible if/when a moving
or generational collector is adopted.

### Plan (informational only — do not start in this slice)
1. Decide on collector evolution direction (mark-compact vs. generational).
2. Switch IR-level pointer locals to use LLVM `gc "statepoint-example"`.
3. Wire `gc.statepoint` insertion into the LLVM emit pass at safepoint sites
   (function entry + loop back-edges already identified by `__newcp_safepoint`).
4. Replace `scan_stack` with `gc.statepoint` stack-map walk.

### Definition of done
- Tracked in `docs/garbage-collection.md`; no action this iteration.

---

## 5. AST / Sema arena allocation

### Evidence
- The recent import-cache change required a `&mut HashMap<String, SemanticModule>`
  threaded through several layers and a `drop(base_sema)` to satisfy the borrow
  checker.
- `SemanticModule` and its symbol vectors are `Clone` and are cloned into the
  cache by value.

### Status of original claim
Accurate as a long-term direction, low priority today.

### Plan
1. Prototype in a branch: introduce `bumpalo::Bump` owned by the driver, lift
   AST node types to `&'arena Node`, and change `SemanticSymbol` storage to
   `&'arena SemanticSymbol`.
2. Measure: parse + sema time on the full `Mod/` corpus before/after.
3. Land only if (a) the borrow patterns visibly simplify and (b) there is no
   regression on existing tests.

### Definition of done
- Decision recorded (adopt or defer) with measurement numbers in this file.

---

## 6. `unwrap()` / `expect()` audit in IR lowering

### Status update
Started on 2026-05-05.

Implemented in code:
- Removed the two production `unwrap()` sites from `newcp-ir/src/lower.rs`.
- Single-character string literals now lower without a second unchecked
  `chars().next()` call.
- The non-void procedure result-slot load now uses an explicit invariant check
  plus trap fallback instead of panicking.
- Replaced the last production `unreachable!()` in `lower_binary` with an
  explicit invariant fallback.
- `newcp-ir` now denies `clippy::unwrap_used` and `clippy::expect_used` in
  non-test code at the crate root, so new production unwrap/expect regressions
  fail linting immediately.

Still pending:
- Decide whether the remaining invariant fallback should become a structured
  lowering diagnostic once item 1 lands.

### Evidence
- Direct grep over `src/newcp-ir/src/lower.rs` shows
  [line 413](src/newcp-ir/src/lower.rs#L413) `inner.chars().next().unwrap()`
  and [line 2100](src/newcp-ir/src/lower.rs#L2100) `result_slot.unwrap()`.
- These are isolated, but each is a panic path on malformed input that sema
  may not yet catch.

### Plan
1. Keep production `newcp-ir` code free of `unwrap()` / `expect()` calls.
2. If a future invariant truly cannot be recovered from, prefer
  `debug_assert!` plus an explicit IR fallback over a panic.
3. Revisit the trap fallback after item 1 if you want IR lowering to surface a
  structured diagnostic instead of an internal trap.

### Definition of done
- Production `newcp-ir/src` contains no `unwrap()`, `expect()`, or
  `unreachable!()` calls.
- The crate-level non-test deny for `clippy::unwrap_used` and
  `clippy::expect_used` remains in place.

---

## Suggested execution order

1. **Item 3** (`mem2reg` wiring) — smallest, highest readability win for
   `dump-llvm` output, unblocks tighter codegen tests.
2. **Item 2** (cyclic type protection) — small, removes a latent crash.
3. **Item 1** (diagnostic propagation) — moderate; touches three crates but
   has a clear shape.
4. **Item 6** (unwrap audit) — naturally falls out of item 1.
5. **Item 5** (arena) — branch experiment, land conditionally.
6. **Item 4** (precise GC) — defer until collector strategy is chosen.
