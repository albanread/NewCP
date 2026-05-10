# NewCP Test Strategy — language-spec coverage

Drafted against the CP Language Report's section structure (Stan Templ /
Niklaus Wirth's "Component Pascal Language Report" — the spec BlackBox
shipped with), prioritized by leverage given the bug patterns observed
during framework module ports.

## The premise

Three observations drive the design:

1. **Every bug we hit was an obvious feature combination no probe touched.**
   Not subtle. So the most valuable test infrastructure is **systematic
   enumeration** of feature interactions, not depth-search of any one
   feature.
2. **The optimizer hides bugs.** The Ports inner-`Draw` `$len` mismatch
   was latent for who knows how long because LLVM DCE ate it. So the
   test harness should run codegen with optimizations disabled.
3. **We have no IDE to dogfood with.** BlackBox got away with no
   regression suite because its IDE *was* the test. We don't.
   Systematic synthetic coverage matters more for us than it did for
   them.

## Five tiers

### Tier 1 — Lexical & syntactic acceptance (CP §1-3)

**Purpose**: confirm the parser accepts every well-formed surface
construct and rejects obvious malformations.

- **Positive corpus**: one tiny fixture per syntactic form. Number
  literals (decimal, hex, hex with type suffix), CHAR literals (ASCII,
  UTF, `nX`), STRING literals (CHAR vs SHORTCHAR contexts), every
  keyword in every context.
- **Negative corpus**: malformed identifiers, reserved-word-as-identifier,
  unterminated string, missing semicolon, unmatched END, malformed type
  expression — each paired with an expected diagnostic substring.

**Cheap (~50 fixtures), bug-class-wide coverage.** Most are 5-line
modules.

### Tier 2 — Type system & sema (CP §4-7, §10)

**Purpose**: every declaration shape × every type kind, assignment
compatibility, scope rules, override rules.

The Cartesian product that matters:

| Axis              | Values |
|-------------------|--------|
| Type kind         | scalar (each builtin), fixed array, multi-dim array, open array, POINTER TO record, POINTER TO array, untagged record, untagged pointer, procedure type, ANYPTR, ANYREC, SET |
| Declaration site  | module-level VAR, module-level CONST, procedure local, parameter, record field, array element |
| Visibility        | hidden, `*` export, `-` read-only export |
| Scope rule        | forward reference within record, forward reference at module level, qualified-import lookup, alias-of-alias |

Plus a focused **assignment-compatibility matrix**: rows = LHS type,
columns = RHS expression kind, cells = "ok" / "expected sema error". §9.1
enumerates the rules; the matrix can be generated from a small DSL.

**Negative-test corpus** is where this tier earns its keep. Each
tier-1/2 sema rule we ship gets a fixture that *would* compile if the
rule weren't enforced, plus an expected error substring.

### Tier 3 — Expressions (CP §8)

**Purpose**: every operator on every applicable operand type, edge cases.

Per-operator probe template (8.2 ordering):

- **Arithmetic** (`+`, `-`, `*`, `/`, `DIV`, `MOD`, unary `-`): each
  numeric type, mixed-type promotion, overflow at INTSHORT / INTEGER /
  LONGINT boundaries, `MIN/MAX` interactions, `DIV/MOD` with negative
  operands (CP semantics: floored, unlike C).
- **Relational** (`=`, `#`, `<`, `<=`, `>`, `>=`): each comparable type,
  including `CHAR vs CHAR`, `POINTER vs NIL`, `SET vs SET` (subset
  relations).
- **Logical** (`OR`, `&`, `~`): short-circuit semantics, mixed with
  side-effects (function calls in operands).
- **Set** (`+`, `-`, `*`, `/`, `IN`): SET(32) construction, range
  syntax `{0..5, 7}`, set-membership.
- **Type test** (`IS`): on `Ptr`, on `ANYPTR`, on record values inside
  `WITH`, cross-module, multi-level inheritance.
- **Type guard** (`expr(T)`): pointer narrowing, ANYPTR narrowing,
  record narrowing inside `WITH`.

**Designator forms** (8.4): qualified import, nested field, multi-dim
index, dereference, type guard mid-chain, call-as-designator-operand.

**Constant folding** at every operator — confirm constant expressions of
each kind survive sema and emit as literal values.

### Tier 4 — Statements (CP §9)

**Purpose**: every statement form × every control-flow shape.

The matrix here is small and finite:

- **Assignment**: scalar / record / fixed array / open array (via
  `COPY`) / pointer / procedure value. Record-by-value copy semantics
  (the bug we just fixed — make this a regression matrix).
- **Procedure call**: direct / method / indirect / super / nested with
  upvalues / forward-declared. Cross matrix with parameter modes.
- **IF / ELSIF / ELSE**: dead branches, constant condition folding.
- **CASE**: INTEGER ranges, CHAR ranges, sparse vs dense, ELSE,
  fallthrough rejection.
- **WHILE / REPEAT**: break-equivalent via flag, nested.
- **FOR**: positive step, negative step, step that doesn't divide range
  evenly, loop-variable scoping after exit.
- **LOOP / EXIT**: nested loops, EXIT from inner only.
- **WITH**: single arm, multi-arm, ELSE arm, narrowing of receiver vs
  local vs param vs imported, fall-through to outer scope after END.
- **RETURN**: from middle of function, from inside WITH, from inside
  LOOP, dead-code-after-return.

### Tier 5 — Procedures & OO (CP §10) — *this is where the bugs have been*

This is the tier the recent bugs lived in, so the most aggressive
enumeration goes here. The systematic matrix:

| Axis             | Values |
|------------------|--------|
| Receiver form    | `(b: BoxDesc)` value-style, `(b: Box)` pointer-alias, `(VAR b: BoxDesc)`, none |
| Record flavor    | plain (none), `EXTENSIBLE`, `ABSTRACT`, `LIMITED`, plus base/no-base |
| Method flavor    | `NEW`, `EXTENSIBLE`, `ABSTRACT`, `EMPTY`, override-without-NEW |
| Param mode       | value, `IN`, `VAR`, `OUT` |
| Param type       | scalar, record, fixed array, open array, POINTER TO record, POINTER TO array, procedure type, ANYPTR |
| Dispatch site    | direct call, method on local, method on field, method on element, method on temporary, super call |
| Module boundary  | same-module / cross-module receiver / cross-module method definition |

Most cells reduce to a generated probe of the form "call this method,
check the result." A code-generator that emits a probe per cell, plus
a runner, plus a manifest of expected results, would produce 500–1000
probes from a one-page DSL. We don't run them all on every CI commit —
a sampled subset on every push, the full matrix on a nightly schedule.

Plus specific high-value cases that aren't pure Cartesian:

- **Override correctness**: subclass overrides → vtable slot stable,
  super call lands in base.
- **Cross-module override**: subclass in module A overrides method in
  module B. Both directions of recompile.
- **Abstract method coverage**: concrete subclass must implement every
  abstract method (sema enforced).
- **EMPTY method semantics**: callable, no body emitted, override
  allowed.
- **Nested procedures**: upvalue capture of every variable kind,
  open-array upvalues (the `$len` companion bug), recursive nested
  call.

### Tier 6 — Modules (CP §11)

- Import / re-export / qualified lookup / `IMPORT name := module`
  aliasing.
- Initialization order: importer's `BEGIN ... END` runs after
  importee's. Probe this with mutation observers.
- Cyclic import — sema rejection.
- Multi-module program: probe a 3-module dependency chain end-to-end.

### Tier 7 — SYSTEM (CP §12)

The unsafe surface deserves its own slice because errors here are
silent.

- `VAL`, `ADR`, `BIT`, `GET`, `PUT`, `MOVE`, `LSH`, `ROT`.
- Each with the type combinations we currently support, plus negative
  tests for the ones we don't (sema must reject).

### Tier 8 — Runtime traps & GC

Not in the language report but load-bearing:

- ASSERT failure → trap with the right number.
- NIL dereference, array index out of bounds, type guard failure,
  narrowing failure, division by zero — each produces a documented
  trap.
- GC: stack roots survive a collection mid-procedure, finalizers fire,
  weak references (if/when added).
- Stack unwind through nested procedures.

## Infrastructure

What we'd build to support this:

1. **Probe generator** (Rust binary in the workspace). Reads a small
   DSL describing one row in the matrix, emits a `.cp` fixture + a Rust
   test entry. Lets us go from "I want to test (receiver=pointer-alias,
   mode=VAR, type=record, dispatch=super)" to a runnable probe in
   seconds.

2. **Negative-test runner**. The existing `loader_error` helper handles
   this; formalize a `Mod/Tests/Negative/` directory with `.cp` fixtures
   + a sibling `.expected` text file containing diagnostic substrings.
   One Rust test iterates the directory.

3. **Optimization-disabled CI lane**. Run the full test suite with
   `--opt none` on every push, alongside the existing `default` lane.
   The Draw inner-proc bug class will trip immediately.

4. **Coverage tracking, lightweight**. Tag each probe with the
   language-report § it exercises. Render a coverage matrix as a CI
   artifact (every § cell colored green/yellow/red by probe count).
   Don't gate on it — just make blind spots visible.

5. **Obx corpus as the integration tier** — eventually. Once Dialog /
   Views / TextModels / Controllers etc. land, "compile every
   ObxFoo.cp and exercise its `Run`/`Sample` entry point" becomes a
   real bar. That's many ports away; keep it on the wish list.

## Phasing

What we'd do in order, given finite time:

1. **First**: Tier 5's procedure × method × dispatch matrix. That's
   where the recent bugs lived. Probe generator + ~200 cells. Probably
   1–2 days of work, immediately catches the next round of bugs in the
   same class.

2. **Then**: the optimization-disabled CI lane. Half a day. Tells us
   how many other Ports-Draw-style latent bugs are sitting in the
   existing modules.

3. **Then**: Tier 2's negative-test corpus, structured by § of the
   spec. Cheap to write (5-line fixtures), high signal — every sema
   rule we have gets a "you must reject this" guardrail. ~2 days.

4. **Then**: Tier 3's expression matrix. The operator rules in CP §8
   are dense; we already mostly cover them but the matrix would
   surface gaps in integer-promotion and constant-folding paths.

5. **In parallel with port work**: Tier 8's trap / GC corner cases.
   Best written when each runtime change lands so the regression
   catches the specific failure mode.

6. **Wishlist**: Obx as integration once enough framework is up.

## Maintenance principles

- **Probe-per-cell, generated from a manifest.** Adding a new language
  feature means adding rows to the manifest, not handwriting N probes.
- **Tag every probe with the spec § it covers.** Coverage report is
  the artifact.
- **One regression test per fixed bug, but write the matrix row too.**
  The bugs we've fixed (pointer-alias receivers, value-by-value
  params, plain-record dispatch) each correspond to a matrix row that
  was empty. Backfill those rows so the next bug in the neighborhood
  is impossible.
- **Don't gate on coverage.** Gate on green tests. Coverage is a
  planning tool, not a barrier.

## What this is *not*

- Not property-based / fuzz testing. The bug-class signal isn't there
  — see the rationale above.
- Not formal verification. CP is small enough that the matrix is the
  verification.
- Not BlackBox parity testing. No upstream test corpus to compare
  against; we'd be inventing oracles either way.

---

**TL;DR**: build a probe-generator-driven matrix for Tier 5 (procedures
× OO) first because that's where every recent bug has been; bolt on an
`-O0` CI lane to surface DCE-hidden latent ABI bugs; structure negative
sema fixtures by § of the spec; tag every probe with the rule it tests
so we can see blind spots. Phased over maybe 5 working days. After that,
we'd be operating with much more visibility into what's actually
covered vs hoped-for.

---

## What's landed so far

**Generator + initial probe corpus** (this session).

- `src/newcp-test-matrix/` — `cargo run -p newcp-test-matrix` emits a
  `.cp` fixture per manifest row into `Mod/Tests/Matrix/` plus a single
  `tests/newcp-tests/src/tests/matrix_generated.rs` declaring one
  `#[test]` per cell.
- Manifest format is a flat Rust array of `Probe { module_name,
  test_name, spec_section, description, expected_value, cp_source,
  ignored }`. The `ignored` slot documents a probe that surfaces a
  known compiler/runtime bug — the probe stays in the matrix as the
  regression target, but the suite stays green; un-ignore = the fix
  landed.
- Seeded 15 cells across receiver shapes, parameter modes, type kinds,
  method dispatch sites, and the super-call path. Backfilled every
  bug class fixed earlier in the session (pointer-alias receiver,
  value-mode params, plain-record dispatch, open-array `$len` ABI).

**`-O0` lane via `NEWCP_OPT`** — the loader reads the env var and
threads it through `CodegenOptions::opt_level`. `NEWCP_OPT=none cargo
test` runs the whole suite unoptimized, surfacing ABI mismatches DCE
would otherwise hide.

### What the new infrastructure caught on its first run

- **Matrix probe `M_Method_On_RecordField`**: `NEW(o.inner)` where
  `inner` is a record-field pointer trips IR codegen with
  `Instr::New: unknown record type opaque:new-ptr`. Filed as item 14
  in `deferred_fixes.md`; probe is `#[ignore]`-flagged with the bug
  reason.
- **`-O0` lane found 8 latent failures**: all in `Strings` (and `Math`
  via transitive import). The LLVM verifier reports
  `Call parameter type does not match function signature! i32 143 / i8`
  against `Strings$g1$RealToShortStrForm` — CHAR (`i32`) vs SHORTCHAR
  (`i8`) at a call boundary. Filed as item 15 in `deferred_fixes.md`.

The matrix is doing its job — every "obvious feature combination" we
add either passes (cementing the contract) or fails (revealing a
defect we didn't know about).

## Next steps from here

1. Fix `deferred_fixes.md` items 14 and 15, un-ignore the relevant
   matrix cells, and add the `NEWCP_OPT=none` lane to CI alongside
   the default lane.
2. Grow the manifest. The 15 seeded cells cover ~20% of the
   interesting tier-5 surface. Easy wins:
   - Cross-module super calls (we have the same-module shape).
   - VAR/IN/OUT on POINTER TO record params.
   - Method calls on a temporary (call returning a record pointer).
   - `ANYPTR` parameter and type-guard narrowing.
   - Receiver formal mutated by a nested proc upvalue.
3. Start the negative-test corpus (tier 2). Same probe-generator
   pattern with `expected_diagnostic` instead of `expected_value`;
   the existing `loader_error` helper handles the assertion.
