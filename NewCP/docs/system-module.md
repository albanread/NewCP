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

### 7.1 Sema (`newcp-sema`)

1. Recognise `IMPORT SYSTEM` specially. Bind it to a built-in `Module` value
   whose decl table contains:
   - All P1/P2 intrinsic names from §3 / §4 as `Procedure` symbols, each
     flagged with an `Intrinsic::SystemAdr`, `Intrinsic::SystemLsh`, … kind.
   - The signature checking for these is custom, not table-driven (e.g. `VAL`
     takes a *type* as its first argument, not a value; `GETREG`'s first
     argument must be an integer constant). Implement them as match arms on
     the intrinsic kind.
2. Allow system flags in the relevant grammar positions **only** when the
   current module's import list contains `SYSTEM`. Reject otherwise with
   `"system flag '%s' requires IMPORT SYSTEM"`.
3. Reject every ✗-tier name and flag from §3/§4/§5 with a clear "not
   supported" diagnostic. Do not silently parse and ignore.

### 7.2 Codegen (`newcp-llvm`)

1. Add `enum SystemIntrinsic { Adr, Val, Lsh, Rot, Typ, Bit, Get, Put, Move, NewSys }`
   and a `lower_system_intrinsic` dispatch in the call-emission path. Each arm
   is small and direct (see §3 / §4 columns).
2. Emit untagged records as plain LLVM struct types **without** a `TypeDesc`
   global. Emit `ptroffs` only for tagged records.
3. The `align2` / `align8` / `noalign` / `union` flags map directly to LLVM
   struct field offsets; verify with `target_data.offset_of_element` after
   construction.

### 7.3 Runtime (`newcp-runtime`)

1. Add `__newcp_sys_new(n: usize) -> *mut u8` that returns memory from
   `std::alloc::alloc(Layout::from_size_align(n, 16).unwrap())`. The pointer
   is **not** registered with the GC and **not** scanned. Document at the
   call site that the buffer is leaked for the lifetime of the process unless
   an explicit free helper is added later.
2. Provide Rust replacements for the BlackBox `[code]` procedures we actually
   need. The full audit lives in [llvm-codegen-design.md](llvm-codegen-design.md);
   the unconditional set is `Math.Sqrt` (→ `f64::sqrt`), `Kernel.Erase`
   (→ `memset`), and a small handful of `Math` trig functions (→ `f64::sin`
   etc.). All of these become normal exported procedures in the Rust-hosted
   `Math` / `Kernel` modules; the source `MODULE Math` becomes a façade until
   we can compile real CP `Math`.

### 7.4 Diagnostics catalogue

Add to the diagnostics table (one entry per refusal):

| Code | Message |
| --- | --- |
| `E_SYSTEM_NOT_IMPORTED` | "use of SYSTEM.X requires IMPORT SYSTEM" |
| `E_SYSTEM_X86_ONLY` | "SYSTEM.X is x86-32 specific and not supported on this target" |
| `E_SYSTEM_GC_PTR_PUN` | "SYSTEM.VAL/PUT/MOVE may not produce a managed pointer; use POINTER [untagged]" |
| `E_SYSTEM_CODE_PROC` | "PROCEDURE [code] (raw byte sequences) is not supported; reimplement as a runtime helper" |
| `E_SYSTEM_INTERFACE_MODULE` | "interface modules (MODULE [\"DllName\"]) are not yet supported" |

## 8. Open questions

- **`SYSTEM.HALT(n)` vs `HALT`**: `HALT` is a *language* built-in (see
  [CP-Lang.odc.md](../../review-md/Docu/CP-Lang.odc.md)), not a `SYSTEM`
  intrinsic. Confirm in the parser tables.
- **`union` records**: are any actually used in the BlackBox sources we plan
  to compile? Grep the Mod tree before promoting from P3 → P1.
- **Cross-module `SYSTEM` taint**: BlackBox does not propagate the
  "implementation-specific" attribute through transitive import. We won't
  either, but a lint that warns about `IMPORT SYSTEM` in core / portable
  modules would be cheap and useful.
