# The `SYSTEM` Pseudo-Module

This document specifies how the NewCP compiler recognises and lowers the
`SYSTEM` module, and which BlackBox `SYSTEM` features we support, defer, or
deliberately refuse.

**Source of truth**: BlackBox's
[Dev/Docu/P-S-I.odc.md](../../review-md/Dev/Docu/P-S-I.odc.md) §"Module SYSTEM"
is the language-level reference for `SYSTEM`. The compiler-internal intrinsic
identifiers used in BlackBox's own codegen live in
[Dev/Mod/CPT.odc.md](../../review-md/Dev/Mod/CPT.odc.md) (`adr`, `val`, `lsh`,
`get`, `getfn`, `sysnewfn`, …) and in [Dev/Mod/CPB.odc.md](../../review-md/Dev/Mod/CPB.odc.md)
(backend dispatch on those names). Keep both at hand when implementing.

## 1. Why `SYSTEM` is special

`SYSTEM` is a **pseudo-module**: there is no `SYSTEM.cp` source file and no
symbol file. The compiler synthesises a built-in symbol table for it whenever
a module writes `IMPORT SYSTEM`. Importing `SYSTEM`:

1. Makes the intrinsics in §3 / §4 visible as qualified names (`SYSTEM.ADR`, …).
2. Unlocks the **system-flag syntax** (`RECORD [untagged] …`,
   `PROCEDURE [code] …`, etc.) for that module only — modules that do not
   import `SYSTEM` may not write a system flag anywhere.
3. Marks the importing module as **patently implementation-specific**. Wirth's
   own commentary on this is quoted in
   [Docu/DTC-Comp.odc.md](../../review-md/Docu/DTC-Comp.odc.md).

`SYSTEM` is not part of Component Pascal proper. Every use is a deliberate
escape from the language's safety guarantees, and our compiler should treat it
as such (clear error messages when used in modules that do not import it; no
implicit imports; never `EXPORT` a `SYSTEM`-typed value across a public boundary).

## 2. Scope and policy for this compiler

We support exactly the subset of `SYSTEM` that the BlackBox sources we intend
to compile actually use. The rest is rejected with a clear "not supported"
diagnostic rather than silently miscompiled.

| Tier | Policy |
| --- | --- |
| P1 | **Must work for the BlackBox bootstrap.** Implement first. |
| P2 | Common in BlackBox sources but not on the critical path. Implement after P1. |
| P3 | Rare or x86-32 specific. Implement only if a source we must compile uses it. |
| ✗  | **Refuse with a diagnostic.** Either unsafe-with-our-GC, or x86-32-only. |

The next two sections classify every intrinsic; §5 does the same for system
flags; §6 spells out lowering details that aren't obvious from the table.

## 3. Function-procedure intrinsics (expressions)

| Name | BlackBox signature | Tier | Lowering |
| --- | --- | --- | --- |
| `ADR(v)` | `(any var) → INTEGER` | P1 | LLVM `ptrtoint` of the variable's address. For arrays/records, address of the first byte. **`INTEGER` is 64-bit in NewCP** (see §6.1). |
| `ADR(P)` | `(P: PROCEDURE) → INTEGER` | P1 | `ptrtoint` of the function symbol. |
| `ADR(T)` | `(T: record type) → INTEGER` | P1 | `ptrtoint` of the type's `TypeDesc` global emitted by codegen. |
| `TYP(v)` | `(record var) → INTEGER` | P2 | Read the block header's `tag` field (mark bit stripped) for heap records; for stack records, the static `TypeDesc` address. |
| `VAL(T, x)` | `(type, any) → T` | P1 | Bitcast / no-op reinterpret. Both operand types must have the **same bit width** in our target ABI; reject otherwise. **Never permit `VAL` to produce a managed pointer from an integer** — see §6.2. |
| `LSH(x, n)` | `(int, int) → typeof x` | P1 | Logical shift: `n > 0` → `shl`; `n < 0` → `lshr` by `-n`. `n` may be a runtime value; emit a `select` on `n`'s sign or two branches. **Operates on `x`'s declared bit width**, not on a fixed 32-bit register. |
| `ROT(x, n)` | `(int, int) → typeof x` | P2 | Lower to LLVM intrinsics `@llvm.fshl`/`@llvm.fshr` over `x`'s bit width. |
| `CC(n)` | `(const int) → BOOLEAN` | ✗ | x86-32 FPU/CPU condition-code probing. Refuse: "SYSTEM.CC is x86-specific and not supported". If a source needs it, replace at the source level. |
| `BIT(a, n)` | `(INTEGER, INTEGER) → BOOLEAN` | P3 | `(load i8 ptr) >> n) & 1`. `a` is an absolute address. Useful for memory-mapped registers; not used by host BlackBox on Windows. |

## 4. Proper-procedure intrinsics (statements)

| Name | BlackBox signature | Tier | Lowering |
| --- | --- | --- | --- |
| `GET(a, v)` | `(INTEGER, basic|ptr|proc var) → ` | P1 | `v := load (inttoptr a)`. Width = `SIZE(v)`. |
| `PUT(a, x)` | `(INTEGER, basic|ptr|proc) → ` | P1 | `store x, (inttoptr a)`. Width = `SIZE(x)`. **Reject if `x` is a managed pointer** (see §6.2). |
| `MOVE(a0, a1, n)` | `(INTEGER, INTEGER, int) → ` | P1 | `@llvm.memmove` of `n` bytes between raw addresses. **Reject** if either region is statically known to overlap a managed allocation containing pointers. |
| `NEW(p, n)` | `(untagged-ptr, int) → ` | P2 | Allocates `n` bytes of **un-traced** memory, sets `p` to point at it. Lower to a runtime call `__newcp_sys_new(n) -> *u8` that returns memory from a separate, GC-ignored arena (e.g. `std::alloc::alloc`). The result is **not** a managed pointer; `p`'s declared type must be `POINTER [untagged] TO …`. |
| `GETREG(n, v)` | `(const int, var) → ` | ✗ | x86-32 register number (EAX=0…EDI=7) is meaningless on x86-64 / AArch64. Refuse with diagnostic. |
| `PUTREG(n, x)` | `(const int, value) → ` | ✗ | Same. |

## 5. System flags

System flags are written in square brackets after the keyword they qualify
(`RECORD [untagged]`, `POINTER [untagged] TO`, `PROCEDURE [code] …`, …). Their
spelling and numeric aliases come from
[P-S-I.odc.md](../../review-md/Dev/Docu/P-S-I.odc.md).

### 5.1 Record / array / pointer flags

| Flag | Numeric | Applies to | Tier | Semantics in NewCP |
| --- | --- | --- | --- | --- |
| `untagged` (1) | 1 | record, array, pointer | P1 | No `TypeDesc` is emitted; the GC ignores variables of this type and refuses to trace through a pointer of this type. `NEW` (the language built-in) is rejected on untagged pointers; only `SYSTEM.NEW` is allowed. Field offsets follow `MIN(4-byte, size)` alignment, padded to a 4-byte multiple. |
| `noalign` (3) | 3 | record | P2 | Untagged + no field alignment; fields are packed at consecutive byte offsets. |
| `align2` (4) | 4 | record | P2 | Untagged + `MIN(2-byte, size)` field alignment. |
| `align8` (6) | 6 | record | P1 | Untagged + `MIN(8-byte, size)` field alignment. **Required for many 64-bit Win32 structs.** |
| `union` (7) | 7 | record | P3 | Untagged + every field at offset 0; size = max(field sizes). C-style union. |

For arrays, only `untagged` is meaningful (one-dimensional open arrays only;
no bounds checks). For pointers, `untagged` requires the pointee to be an
untagged record.

### 5.2 `VAR` parameter flag

| Flag | Numeric | Applies to | Tier | Semantics |
| --- | --- | --- | --- | --- |
| `nil` | 1 | `VAR p: POINTER …` | P2 | The formal accepts `NIL` as actual. Used only at C-interop boundaries. |

### 5.3 Procedure flags

| Flag | Numeric | Tier | Semantics |
| --- | --- | --- | --- |
| `code` | 1 | ✗ | Inline x86-32 byte sequence. Refuse with diagnostic (we are 64-bit and JIT-compiled; raw byte injection is meaningless). The handful of `[code]` procedures in BlackBox (`Math.Sqrt`, `Kernel.Erase`, …) must be re-implemented in Rust runtime helpers. |
| `ccall` | -10 | P2 | C calling convention. Lower to LLVM `ccc`. Only meaningful on procedures that bridge to Rust runtime symbols or to OS / DLL functions. |

### 5.4 Interface-module flag

`MODULE Name ["DllName"];` declares an **interface module** that the BlackBox
compiler maps to a Windows DLL. NewCP will not consume Windows DLL interfaces
in the bootstrap — `WinApi`, `WinNet`, etc. are out of scope until much later.
**Tier ✗**: parse-and-reject with a "interface modules not yet supported"
diagnostic. Per-procedure `["DllName", "ExportName"]` aliases are similarly
rejected.

## 6. Lowering and safety notes

### 6.1 `INTEGER` width

BlackBox 1.x assumes `INTEGER = 32 bits`, so `ADR`/`VAL` round-trip pointers
through `INTEGER` (= 32 bits, sufficient on x86-32). This is broken on 64-bit
targets. NewCP makes `INTEGER` 64-bit so that `SYSTEM.ADR(x): INTEGER` and
`SYSTEM.VAL(POINTER TO T, addr)` continue to work bit-correctly. Source code
that depends on `INTEGER = 32 bits` (e.g. masking with `MAX(INTEGER)`) must be
audited; this is documented separately in
[component-pascal-language-and-compiler-notes.md](component-pascal-language-and-compiler-notes.md).

### 6.2 GC-pointer corruption

The BlackBox manual is explicit:

> Never use `VAL` (or `PUT` or `MOVE`) to assign a value to a BlackBox pointer.
> Doing this would corrupt the garbage collector, with fatal consequences.

NewCP enforces this at compile time:

- `VAL(P, x)` where `P` is a **managed** pointer type (i.e. `POINTER TO T`
  *without* `[untagged]`) is rejected.
- `PUT(a, x)` where `x` has managed-pointer type is rejected.
- `MOVE` is allowed unconditionally (sources rely on it for byte-blits), but a
  warning is emitted if the destination region statically overlaps a known
  pointer-bearing record.

### 6.3 `SYSTEM.NEW` versus the language `NEW`

| Built-in | Tracing | Block-header | Allocator |
| --- | --- | --- | --- |
| `NEW(p)` / `NEW(p, len)` | Yes — `TypeDesc` recorded in block header | `BlockHeader { tag: TypeDesc*, … }` | `__newcp_new_rec` (managed cluster heap) |
| `SYSTEM.NEW(p, n)` | **No** — `p` must be `POINTER [untagged]` | None | `__newcp_sys_new(n)` (separate Rust-owned arena, never scanned) |

The two arenas must be address-disjoint, otherwise the conservative stack
scanner could mistake a `SYSTEM.NEW`'d block for a managed object. Easiest
implementation: `__newcp_sys_new` calls `std::alloc::alloc`; the conservative
scan only resolves addresses that fall inside a registered cluster.

There is also no `SYSTEM.DISPOSE`. BlackBox's `SYSTEM.NEW` blocks are
reclaimed exclusively by an explicit runtime call (or, in practice, are leaked
for the lifetime of the loaded module). We will start with **no
deallocation API**; revisit if a real source needs one.

### 6.4 Untagged records crossing the GC boundary

A `POINTER [untagged] TO Rec` is opaque to the GC; the GC will not follow it
even if `Rec` itself has fields whose Component Pascal types are managed
pointers. Enforce at type-check time: an untagged record may not contain any
managed-pointer field. (BlackBox does not enforce this, but every BlackBox
source treats untagged records as plain C-style structs of basic types.)

### 6.5 `LSH` semantics

BlackBox's `LSH` shifts within `x`'s declared width and is a **logical** shift
in both directions (filling with zero on right shift). Component Pascal also
has `ASR` for arithmetic right shift, so `LSH(x, -n)` truly is `x >> n` as
unsigned. Lower to LLVM `lshr` for negative `n`, `shl` for positive.

### 6.6 `MOVE` overlap

BlackBox documents `MOVE` as `M[a1..a1+n-1] := M[a0..a0+n-1]` without
specifying overlap behaviour. Both `memcpy` and `memmove` see use in real
sources; lower to `@llvm.memmove` to guarantee correctness when regions overlap.

## 7. Implementation plan

This section is the authoritative task list. Each work item names the exact
file(s) to touch, the data structure or function to add/change, and what
"done" looks like. Items are ordered so that each one compiles on its own —
you can commit after every item without breaking the build.

Crate locations:
- `newcp-parser`  → `src/newcp-parser/src/lib.rs`
- `newcp-sema`    → `src/newcp-sema/src/lib.rs`
- `newcp-ir`      → `src/newcp-ir/src/{types,ir,lower}.rs`
- `newcp-llvm`    → `src/newcp-llvm/src/lib.rs`
- `newcp-runtime` → `src/newcp-runtime/src/{gc,lib}.rs`

---

### Phase 1 — Parser: system-flag syntax

**Goal**: the parser accepts (but does not yet enforce) system-flag brackets
wherever BlackBox allows them. All flags are parsed into the AST as `Option<SysFlag>` fields so that sema and codegen can act on them. No semantic validation happens here.

#### 1a — Add `SysFlag` to the parser AST

In `newcp-parser/src/lib.rs`:

```rust
/// A system flag written as `[ident]` or `[integer]` inside a type or
/// procedure declaration. Only legal when the containing module imports SYSTEM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SysFlag {
    Named(String),     // e.g. [untagged], [ccall]
    Numeric(i64),      // e.g. [-10]
}
```

Add `sys_flag: Option<SysFlag>` to:
- `TypeExpr::Record { ... }` — replaces the existing `RecordFlavor` for untagged records (keep `RecordFlavor` for ABSTRACT/EXTENSIBLE/LIMITED; add `sys_flag` alongside it)
- `TypeExpr::Array { ... }` — for `ARRAY [untagged] OF T`
- `TypeExpr::Pointer { ... }` — for `POINTER [untagged] TO T`
- `FPSection { ... }` — for `VAR [nil] p: T`
- `ProcedureHeading { ... }` — for `PROCEDURE [ccall] F` (alongside the existing `MethodAttributes`)

#### 1b — Parse `[SysFlag]` in the right positions

In the parser methods that produce each of the AST nodes above, attempt to
parse an optional `[` *flag* `]` immediately after the keyword that opens the
production:

- After `RECORD` keyword, before any `(BaseType)` or field list
- After `ARRAY` keyword, before dimension expressions
- After `POINTER` keyword, before `TO`
- After `VAR` / `IN` / `OUT` in formal parameter sections
- After `PROCEDURE` keyword in procedure declarations, before the receiver or name

If a `[` is present but `IMPORT SYSTEM` is not in the module header, the
parser records the flag in the AST and also attaches an error noting it will
be rejected in sema. (This keeps parse errors and semantic errors separate,
matching the pipeline design.)

**Done when**: all existing parser tests still pass, and a new
`parse_sys_flag_*` battery passes for each of the five positions.

---

### Phase 2 — Sema: SYSTEM intrinsics and flag validation

**Goal**: sema knows about every SYSTEM intrinsic and all system flags, emits
structured diagnostics on misuse, and annotates the `SemanticModule` output
with enough information for the IR lowering phase.

#### 2a — Add `SystemIntrinsic` to `BuiltinProc`

In `newcp-sema/src/lib.rs`, extend `BuiltinProc`:

```rust
pub enum BuiltinProc {
    // existing entries ...

    // SYSTEM intrinsics — function procedures (return a value)
    SystemAdr,   // SYSTEM.ADR(v) or SYSTEM.ADR(P) or SYSTEM.ADR(T)
    SystemVal,   // SYSTEM.VAL(T, x)
    SystemLsh,   // SYSTEM.LSH(x, n)
    SystemRot,   // SYSTEM.ROT(x, n)
    SystemTyp,   // SYSTEM.TYP(v)
    SystemBit,   // SYSTEM.BIT(a, n)         -- P3, but parse+reject cleanly

    // SYSTEM intrinsics — proper procedures (statement only)
    SystemGet,   // SYSTEM.GET(a, v)
    SystemPut,   // SYSTEM.PUT(a, x)
    SystemMove,  // SYSTEM.MOVE(a0, a1, n)
    SystemNew,   // SYSTEM.NEW(p, n)
    SystemGetReg, // SYSTEM.GETREG -- ✗, parsed then rejected
    SystemPutReg, // SYSTEM.PUTREG -- ✗, parsed then rejected
}
```

In `BuiltinProc::name`, map each variant to the string the resolver will match.

#### 2b — Recognise `IMPORT SYSTEM` and populate the built-in scope

Add `has_system_import: bool` to `Analyzer`. In `collect_module_symbols`:

```rust
let has_system = self.module.imports.iter().any(|i| i.name == "SYSTEM");
self.has_system_import = has_system;
```

When `has_system` is true, insert synthetic `SemanticSymbol` entries for all
P1/P2 intrinsics into `module_symbols` under module-qualification `"SYSTEM"`.
Use `SymbolKind::Procedure` with `declared_type = Some(SemanticType::BuiltinProc(SystemAdr))`, etc.

For the ✗ intrinsics (`GETREG`, `PUTREG`, `CC`) also insert entries, but
marked so the call validator will reject them with `E_SYSTEM_X86_ONLY`.

#### 2c — System-flag validation in type checking

Add a helper:

```rust
fn check_sys_flag(&self, flag: &Option<SysFlag>, ctx: &str) -> Option<NormalizedSysFlag>
```

Where `NormalizedSysFlag` is an enum of the meaningful variants:

```rust
pub enum NormalizedSysFlag {
    Untagged,
    NoAlign,
    Align2,
    Align8,
    Union,
    NilParam,
    CCall,
}
```

`check_sys_flag` returns `None` and pushes `E_SYSTEM_NOT_IMPORTED` if
`has_system_import` is false. Returns `None` and pushes `E_SYSTEM_CODE_PROC`
for `[code]` or `[1]` on a procedure. Returns `None` and pushes
`E_SYSTEM_INTERFACE_MODULE` for string-valued flags on `MODULE` or
per-procedure. For valid flags it returns `Some(NormalizedSysFlag::…)`.

#### 2d — Propagate flags into `SemanticType`

Add the untagged / alignment concept to `SemanticType`:

```rust
pub enum RecordLayout {
    Tagged,                 // normal managed record
    Untagged,               // [untagged], MIN(4, size) alignment
    UntaggedNoAlign,        // [noalign]
    UntaggedAlign2,         // [align2]
    UntaggedAlign8,         // [align8]
    Union,                  // [union]
}

pub enum SemanticType {
    // ... existing ...
    Record {
        flavor: Option<RecordFlavor>,
        layout: RecordLayout,       // NEW — was implicit "Tagged"
        base: Option<Box<SemanticType>>,
        fields: Vec<FieldType>,
        methods: Vec<MethodType>,
    },
    Pointer {
        target: Box<SemanticType>,
        untagged: bool,             // NEW — POINTER [untagged] TO T
    },
    Array {
        lengths: Vec<String>,
        element_type: Box<SemanticType>,
        untagged: bool,             // NEW — ARRAY [untagged] OF T
    },
}
```

Enforce the GC-pointer-corruption rules (§6.2 of the design doc) here:
when `RecordLayout != Tagged`, reject any field whose type is a managed
pointer. When `Pointer { untagged: true }`, check that the pointee is also
`RecordLayout != Tagged`.

#### 2e — Call-site validation for SYSTEM intrinsics

In the expression/statement resolver, when the callee resolves to a
`BuiltinProc::System*` variant, dispatch to a dedicated validator:

```
fn validate_system_call(intrinsic, args, result_type_hint) -> SemanticType
```

Per-intrinsic checks (non-exhaustive):
- `SystemAdr`: one argument, variable/procedure/type; result type is `INTEGER`.
- `SystemVal`: exactly two args; first must be a *type expression*; both types
  must have the same bit-width at the target ABI; must not produce a managed pointer.
- `SystemLsh` / `SystemRot`: two args, both integer types; result = type of arg 1.
- `SystemGet`: two args; first must be `INTEGER`; second must be a `VAR` variable
  of basic/pointer/procedure type; check for managed pointer in result.
- `SystemPut`: two args; first `INTEGER`; second any basic type; reject managed pointer.
- `SystemMove`: three args; all `INTEGER`/integer.
- `SystemNew`: two args; first must be `POINTER [untagged]`; second integer.
- `SystemGetReg` / `SystemPutReg` / `SystemBit` (P3): emit `E_SYSTEM_X86_ONLY`.
- `SystemTyp` (P2): one arg, must be a record-type variable; result `INTEGER`.

**Done when**: the five diagnostics from §7.4 all fire correctly on test
modules, and all intrinsics emit the right result types into `SemanticModule`.

---

### Phase 3 — IR: untagged types and SYSTEM instructions

**Goal**: the IR can represent every SYSTEM construct, ready for the LLVM
backend to consume. The existing `AddrOf`, `BitCast`, and `MemCopy`
instructions already cover three intrinsics; this phase fills the gaps.

#### 3a — Add `RecordLayout` to `IrType`

In `newcp-ir/src/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordLayout { Tagged, Untagged, UntaggedNoAlign, UntaggedAlign2, UntaggedAlign8, Union }

pub enum IrType {
    // existing entries ...

    /// Untagged record — C-struct equivalent. No TypeDesc emitted.
    /// `layout` determines field alignment rules.
    UntaggedRecord { name: String, layout: RecordLayout },

    /// Untagged pointer — not traced by the GC.
    UntaggedPtr(Box<IrType>),
}
```

Add a `render()` arm for each new variant.

#### 3b — Add the missing SYSTEM instructions

In `newcp-ir/src/ir.rs`, add to `Instr`:

```rust
/// `v := load_raw addr, ty`  — SYSTEM.GET: load from an integer address.
/// `addr` is always i64 (INTEGER on 64-bit target).
LoadRaw { dst: TempId, addr: IrValue, ty: IrType },

/// `store_raw addr, value`  — SYSTEM.PUT: store to an integer address.
StoreRaw { addr: IrValue, value: IrValue },

/// `t = lsh x, n`  — SYSTEM.LSH: logical shift (left if n > 0, right if n < 0).
/// n is a signed value known at runtime; codegen emits a select or two arms.
Lsh { dst: TempId, value: IrValue, shift: IrValue, ty: IrType },

/// `t = rot x, n`  — SYSTEM.ROT: rotation (left if n > 0).
Rot { dst: TempId, value: IrValue, shift: IrValue, ty: IrType },

/// `t = typ v`  — SYSTEM.TYP: read the type-tag word of a managed record.
/// Returns i64 (the TypeDesc pointer as integer, mark bit stripped).
TypTag { dst: TempId, record_ptr: IrValue },

/// `t = call_sys_new n`  — SYSTEM.NEW: allocate n bytes of untraced memory.
SysNew { dst: TempId, size: IrValue },
```

Note: `AddrOf` already handles `SYSTEM.ADR`; `BitCast` handles `SYSTEM.VAL`;
`MemCopy` handles `SYSTEM.MOVE`. They need no changes.

Add `render()` arms for each new variant.

#### 3c — Untagged layout helpers in `lower.rs`

In `newcp-ir/src/lower.rs` (or a new `src/layout.rs`), add:

```rust
/// Computes byte offsets of all fields in an untagged record per BlackBox's
/// alignment rules. Used by both sema (SIZE/offset checks) and codegen
/// (LLVM struct type construction).
pub fn untagged_field_offsets(
    fields: &[(String, usize)],  // (name, byte_size) pairs in order
    layout: RecordLayout,
) -> Vec<usize>
```

Alignment rules (from P-S-I.odc.md):
- `Untagged`: `MIN(4, size)` per field, record padded to 4-byte multiple.
- `UntaggedNoAlign`: consecutive, no padding.
- `UntaggedAlign2`: `MIN(2, size)` per field.
- `UntaggedAlign8`: `MIN(8, size)` per field.
- `Union`: all fields at offset 0, size = `max(field sizes)`.

**Done when**: all existing IR tests pass plus new round-trip tests for
untagged record layout and each new instruction.

---

### Phase 4 — Runtime: `__newcp_sys_new` and code-procedure replacements

**Goal**: the runtime exports everything `SYSTEM`-using modules need that
cannot be inlined by the LLVM backend.

#### 4a — `__newcp_sys_new` in `newcp-runtime/src/gc.rs`

```rust
/// Allocates `n` bytes of **untraced** memory for `SYSTEM.NEW`.
///
/// The returned pointer is NOT registered with the GC, NOT zero-initialised,
/// and NOT freed during the lifetime of the process (there is no `SYSTEM.DISPOSE`).
/// Callers are responsible for initialising every byte before use.
///
/// Uses 16-byte alignment (safe superset of all CP primitive types).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __newcp_sys_new(n: usize) -> *mut u8 {
    if n == 0 { return std::ptr::NonNull::dangling().as_ptr(); }
    let layout = std::alloc::Layout::from_size_align(n, 16)
        .expect("__newcp_sys_new: size overflows alignment");
    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() { std::alloc::handle_alloc_error(layout); }
    ptr
}
```

Key invariants to document:
1. Addresses returned here never fall inside a registered `Cluster`, so the
   conservative stack scan cannot accidentally treat them as managed objects.
2. The GC's `resolve_heap_ptr` already skips non-cluster addresses — no
   change needed to the GC.

#### 4b — Rust replacements for BlackBox `[code]` procedures

Add a new file `newcp-runtime/src/builtins.rs` (and `pub mod builtins;` in
`lib.rs`) with the Rust implementations of the handful of `[code]` procedures
used in BlackBox sources. These are exported with `__newcp_` prefixes so the
compiler can call them as normal procedures once the `MODULE Math` / `MODULE
Kernel` façades redirect to them.

Minimum set for the bootstrap:

| BlackBox symbol | Rust body | Export name |
| --- | --- | --- |
| `Math.Sqrt(x: REAL): REAL` | `x.sqrt()` | `__newcp_math_sqrt` |
| `Math.Sin(x: REAL): REAL` | `x.sin()` | `__newcp_math_sin` |
| `Math.Cos(x: REAL): REAL` | `x.cos()` | `__newcp_math_cos` |
| `Math.Exp(x: REAL): REAL` | `x.exp()` | `__newcp_math_exp` |
| `Math.Ln(x: REAL): REAL` | `x.ln()` | `__newcp_math_ln` |
| `Kernel.Erase(adr, words: INTEGER)` | `ptr::write_bytes(adr, 0, words*4)` | `__newcp_kernel_erase` |

None of these touch managed memory; no GC interaction.

**Done when**: `cargo test -p newcp-runtime --lib` still passes and the six
exports are visible in `nm --defined-only` output.

---

### Phase 5 — LLVM codegen: lower SYSTEM IR to LLVM IR

**Goal**: the LLVM backend can consume every SYSTEM IR node and produce
correct LLVM IR. This phase is last because it depends on all earlier phases
being stable.

#### 5a — Untagged record types

In `newcp-llvm/src/lib.rs`, when building an LLVM struct type for a record:

- **Tagged** record: emit `%T = type { i64, [N x i8] }` (block-header + payload) and a `@T.TypeDesc` global as today.
- **Untagged** record: emit `%T = type { f1_ty, f2_ty, … }` using the field
  offsets from `untagged_field_offsets()`. No `TypeDesc` global. `Union`
  records emit as `%T = type { [size x i8] }` since LLVM has no union type.

#### 5b — Dispatch table for SYSTEM IR instructions

Add `fn lower_instr_system(instr: &Instr, builder: &LLVMBuilder) -> LLVMValue`
with match arms for each new instruction type:

| Instruction | LLVM emission |
| --- | --- |
| `LoadRaw { addr, ty }` | `inttoptr addr to ptr`, then `load ty, ptr` |
| `StoreRaw { addr, value }` | `inttoptr addr to ptr`, then `store value, ptr` |
| `Lsh { value, shift, ty }` | `select (icmp slt shift, 0) (lshr value, neg shift) (shl value, shift)` |
| `Rot { value, shift, ty }` | `call @llvm.fshl.iN(value, value, urem shift, N)` (left); fshr for right |
| `TypTag { record_ptr }` | `load i64, ptr (record_ptr - sizeof(BlockHeader))` then mask mark bit |
| `SysNew { size }` | `call @__newcp_sys_new(size)` |
| `AddrOf { sym }` | `ptrtoint sym to i64` (already present, confirm i64 not i32) |
| `BitCast { value, ty }` | `bitcast value to ty` (already present) |
| `MemCopy { dst, src, len }` | `call @llvm.memmove.p0.p0.i64(…)` (already present) |

#### 5c — Untagged pointer handling

When a pointer type is `IrType::UntaggedPtr`, emit as `ptr` (LLVM opaque
pointer) but suppress the nil-check guard that would normally precede a
dereference of a managed `IrType::Ptr`. Also suppress `TypeDesc` field
emission in any `Call` lowering that would normally add a GC root.

**Done when**: a round-trip test module that uses every P1 SYSTEM intrinsic
compiles to valid LLVM IR and the IR verifier passes.

---

### Phase 6 — Integration tests

In `tests/newcp-tests/` (or a new `tests/newcp-compat-tests/`), add test
modules that exercise the full pipeline:

| Test module | What it exercises |
| --- | --- |
| `SystemAdr.cp` | `SYSTEM.ADR` of a local, a global, a procedure, a type |
| `SystemVal.cp` | `SYSTEM.VAL` for integer↔real reinterpret; rejection of managed-pointer VAL |
| `SystemLsh.cp` | `SYSTEM.LSH` with positive, negative, and runtime-variable shift |
| `SystemGetPut.cp` | `SYSTEM.GET` / `SYSTEM.PUT` round-trip on a stack buffer |
| `SystemMove.cp` | `SYSTEM.MOVE` of a byte range; overlap case |
| `SystemNew.cp` | `SYSTEM.NEW` for an untagged record; untagged pointer dereference |
| `UntaggedRecord.cp` | Untagged record layout for `align8`, `noalign`, `union` |
| `SystemErrors.cp` | All ✗-tier calls produce the correct diagnostics |

Each test has a `.expected` file checked in alongside it, just like the
existing compat tests.

---

### Summary and ordering

| # | Phase | Crates touched | Blocker for |
| --- | --- | --- | --- |
| 1 | Parser: SysFlag AST nodes | `newcp-parser` | 2 |
| 2 | Sema: intrinsics + flag validation | `newcp-sema` | 3 |
| 3 | IR: new instructions + untagged types | `newcp-ir` | 5 |
| 4 | Runtime: `__newcp_sys_new` + builtins | `newcp-runtime` | 5 |
| 5 | LLVM codegen | `newcp-llvm` | 6 |
| 6 | Integration tests | `tests/` | — |

Phases 3 and 4 are independent and can be done in parallel.
All of 1–4 must be complete before phase 5 starts.

## 8. Open questions

- **`SYSTEM.HALT(n)` vs `HALT`**: `HALT` is a *language* built-in (see
  [CP-Lang.odc.md](../../review-md/Docu/CP-Lang.odc.md)), not a `SYSTEM`
  intrinsic. Confirm in the parser tables before adding `HALT` to the `SYSTEM`
  built-in scope.
- **`union` records**: are any actually used in the BlackBox sources we plan
  to compile? `grep -r SYSTEM.union review-md/` before promoting from P3 → P1.
- **Cross-module `SYSTEM` taint**: BlackBox does not propagate the
  "implementation-specific" attribute through transitive import. We won't
  either, but a lint that warns about `IMPORT SYSTEM` in core / portable
  modules would be cheap and useful.
- **`SYSTEM.NEW` deallocation**: if a BlackBox source ever calls a
  `SYSTEM.NEW` allocation and later needs to free it, we will need
  `__newcp_sys_free(ptr, n)`. Audit the sources before dismissing.

