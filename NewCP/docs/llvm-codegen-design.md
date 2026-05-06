# LLVM Code Generation Design

## Status

This document defines the design for `newcp-llvm` and tracks its implementation progress.

Current state:

- `newcp-ir` lowers typed Component Pascal procedures to CFG-shaped IR
- `newcp-ir` supports robust LLVM emission, carrying `SYSTEM`-driven backend shapes (`BitCast`, `LoadRaw`, `StoreRaw`, `SysNew`, etc.), complex control flow, standard intrinsics, multidimensional arrays, and full method dispatch.
- `newcp-llvm` is fully implemented using Inkwell. It emits complete, verifiable LLVM IR and performs inline ORC JIT execution.
- the driver fully exposes `--dump-llvm-ir` and `--run-jit`, powering our regression integration test suite (`tests/newcp-tests/`).
- JIT emission handles inline string literal allocations, recursive functions, type tests (`IS`/`WITH`), nested procedures (lifted to globals), and JIT-friendly `vtable` emission for tagged records.

This document serves as both the original design blueprint and a record of the realized architecture.

## Goals

- lower `newcp-ir::IrModule` into a real LLVM module using Inkwell
- keep LLVM generation as an observable compiler phase with a stable textual dump
- compile procedures to native code through LLVM JIT
- expose compiled exported procedures so the runtime can invoke them
- keep the implementation narrow enough for bring-up, then extend systematically

Success for the first implementation means:

- `dump-llvm <file>` is produced from `newcp-ir`, not from `newcp-parser`
- the generated LLVM module verifies successfully
- a small subset of exported procedures can be looked up from the JIT and invoked
- the emitted LLVM IR is understandable enough to debug codegen errors from the textual dump alone

## Non-goals For The First Slice

- full BlackBox-compatible runtime descriptors
- unloading compiled modules
- optimizing IR aggressively before correctness is established
- full coverage of all IR instructions and all Component Pascal features in one step
- final ORC-based loader integration if MCJIT-style execution is sufficient for initial bring-up

## Phase Boundaries

The LLVM subsystem should be split into explicit stages:

1. LLVM target initialization
2. LLVM type lowering
3. symbol table and global declaration planning
4. procedure declaration pass
5. basic block creation pass
6. instruction emission pass
7. terminator emission pass
8. module verification
9. LLVM IR textual dump
10. JIT materialization and exported symbol lookup

Each stage should have a defined input, output, and failure mode.

## Main Components

The first implementation should likely contain these components:

- `CodegenOptions`
- `CodegenError`
- `CodegenModule`
- `TypeLowerer`
- `GlobalPlanner`
- `ProcedureEmitter`
- `ValueMap`
- `JitModule`

`CodegenOptions` is a plain configuration struct threaded through all stages. It should contain:

- `target_triple: Option<String>` — override the host target; `None` means use native host
- `opt_level: OptLevel` — `None | Less | Default | Aggressive`; first slice should default to `None`
- `emit_debug_info: bool` — reserved for later; must be `false` in the first slice
- `strict_unsupported: bool` — when `true`, any `Unsupported` error is fatal; when `false`, unsupported instructions emit a call to a stub trap instead of aborting compilation (useful for bring-up of modules that exercise unimplemented paths)

`CodegenOptions` must not carry mutable state. It is constructed once by the driver and passed by reference.

`ValueMap` is the per-procedure name resolution context. It wraps:

- `temp_values: HashMap<TempId, BasicValueEnum>` — SSA values produced during emission
- `block_map: HashMap<BlockId, BasicBlock>` — LLVM blocks keyed by IR block identity
- global symbol resolution delegated to the `GlobalPlanner` symbol table

`CodegenModule` is the central coordinating object for one compilation job. It owns:

- the Inkwell `Context` and `Module`
- the `Builder` used throughout emission
- the `GlobalPlanner`'s symbol tables
- the `CodegenOptions` reference

It is created at Stage 2 and consumed at Stage 5 to produce the `CompiledModule`. Nothing inside `CodegenModule` survives into `JitModule`; the JIT stage takes the verified LLVM module by value.

`ProcedureEmitter` holds a `ValueMap` for the duration of one procedure body and clears it between procedures. Keeping `ValueMap` as a distinct type makes the emission context explicit and prevents `temp_values` from leaking across procedure boundaries by accident.

## Architectural Constraints

The design respects five constraints:

1. `newcp-ir` is the controlling input. `newcp-llvm` must not quietly re-derive behavior from the parser or source module surface.
2. Every major compiler phase is observable. LLVM generation must therefore produce both a programmatic result and a stable textual dump.
3. The first runtime target is a 64-bit Windows JIT process (with `pointer` and integer sizes adhering).
4. The IR is an expressive mid-level representation mapping closely to LLVM intrinsics, but retaining Component Pascal semantics (like `TypeCheck`, `SysNew`, etc.).
5. `SYSTEM` raw-address operations leverage 64-bit integer values and address spaces instead of legacy 32-bit x86 assumptions.

## Public API

The current `newcp-llvm` API is too shallow because it only renders placeholder strings. The implementation should move to a small set of explicit entry points:

```rust
pub fn dump_llvm(path: &Path) -> String;
pub fn dump_asm(path: &Path) -> String;

pub fn compile_ir_module(ir_module: &newcp_ir::IrModule) -> Result<CompiledModule, CodegenError>;
pub fn jit_module(compiled: CompiledModule) -> Result<JitModule, CodegenError>;

// Convenience helpers used by the driver; they compose the two steps above.
pub fn compile_from_path(path: &Path) -> Result<CompiledModule, String>;
pub fn jit_from_path(path: &Path) -> Result<JitModule, String>;
```

The two-step split matters: `compile_ir_module` produces a verifiable, dumpable artifact that tests can inspect without running any JIT machinery. `jit_module` then consumes that artifact to materialize native code. Driver convenience helpers exist only to chain the two steps; they should not bypass the IR stage.

Supporting types should look roughly like this:

```rust
pub struct CompiledModule {
	pub module_name: String,
	pub llvm_ir: String,
	pub exported_functions: Vec<ExportedFunction>,
}

pub struct ExportedFunction {
	pub public_name: String,
	pub llvm_name: String,
	pub params: Vec<newcp_ir::IrType>,
	pub ret_ty: newcp_ir::IrType,
}
```

Note on `ExportedFunction` type coupling: embedding `newcp_ir::IrType` directly means any crate that reads `CompiledModule.exported_functions` must also depend on `newcp-ir`. For the driver and test crates this is already the case, so it is acceptable in the first slice. If `newcp-llvm` is ever consumed without `newcp-ir` (e.g., by a loader-only crate), `ExportedFunction` should switch to a flattened representation. That decision is deferred; the current coupling is intentional and documented.

```rust
pub struct JitModule {
	// owns the ORC LLJIT session and the module's JITDylib
}

pub enum CodegenError {
	Parse(String),
	Sema(String),
	Unsupported { stage: &'static str, detail: String },
	Verify(String),
	Jit(String),
}
```

Design rules for the public API:

- `dump_llvm` remains string-oriented because it is a driver-facing inspection surface
- `compile_ir_module` is the real backend entry point and should be the unit-test anchor
- path-based helpers exist only to preserve the driver workflow
- the compiled result carries the exact LLVM IR string used for inspection, so dumping and execution cannot diverge
- JIT symbol lookup should expose exported procedures by stable public name, not raw LLVM names

## Internal Module Structure

The first usable `newcp-llvm` implementation should stay small and explicit. A reasonable layout is:

- `lib.rs`: driver-facing entry points and high-level orchestration
- `types.rs`: `IrType` to LLVM type lowering
- `module.rs`: module creation, target setup, globals, function declarations
- `emit.rs`: procedure, instruction, and terminator emission
- `jit.rs`: ORC LLJIT session management, `JITDylib` lifecycle, and exported symbol lookup
- `error.rs`: `CodegenError`

This is not about creating many files for their own sake. It is about separating three concerns that otherwise get tangled immediately:

- LLVM object setup
- IR-to-LLVM lowering
- JIT ownership and invocation

## End-To-End Pipeline

The code generation pipeline should be a fixed sequence.

### Stage 1: Front-end handoff

Input:

- source path

Output:

- `newcp_ir::IrModule`

Rule:

- `newcp-llvm` must receive typed CFG IR from `newcp-ir::lower_module`

### Stage 2: LLVM context and target setup

Input:

- `IrModule`
- host target information

Output:

- `Context`
- `Module`
- `Builder`

Rule:

- initialize native target and ASM printer once before JIT use

### Stage 3: Module planning

Input:

- `IrModule.globals`
- `IrModule.procedures`
- `IrModule.type_vtables` — map from type name to ordered list of bound-procedure LLVM names (slot order)
- `IrModule.type_bases` — map from type name to base type name (or `None`)

Output:

- a single packed LLVM Struct definition representing all mutable globals (`%Module.Data`)
- declared LLVM globals (the singleton `@Module.Data` and immutable read-only strings/constants)
- pointer offset table array (`@Module.Ptrs`) containing byte offsets into the data struct for GC scanning
- declared LLVM functions
- external symbol declarations for every `ImportRef` name referenced in the module body
- synthesized init function declaration: `@Module.$init` with signature `fn() -> void`
- `@TypeName.vtable` constant globals — one per record type that has at least one bound procedure
- `@TypeName.desc` constant globals — one per record type that has at least one bound procedure
- symbol maps keyed by IR/global name and procedure name

Rule:

- globals **must not** be emitted as scattered standalone LLVM variables because the BlackBox GC (`Kernel.MarkGlobals`) expects exactly one base memory address (`varBase`) and an array of pointer offsets for the entire module.
- declaration must happen before body emission so forward references work naturally
- `@Module.$init` must always be planned, even if the source module body is empty, because the runtime always calls it after loading
- every `ImportRef` that appears in the IR must have a corresponding LLVM `declare` emitted during this stage; any `ImportRef` that was not planned here must fail with `Unsupported` during value lowering, not silently emit a broken reference
- when building `@Module.Ptrs`, include only managed pointer roots; `UntaggedPtr`, untagged records, and raw address values introduced for `SYSTEM` must not appear in the pointer-offset table
- declare `@__newcp_sys_new(i64) -> ptr` whenever `Instr::SysNew` appears in the module; this helper is part of the backend/runtime ABI, not an optional convenience
- vtable and TypeDesc globals are emitted by `declare_type_descs` after all function declarations, so vtable slot entries can reference already-declared function values

### Stage 4: Procedure lowering

Input:

- one `IrProcedure`

Output:

- fully emitted LLVM function body

Rule:

- perform body emission in two passes: block creation first, then instruction/terminator emission

### Stage 5: Verification and inspection artifact creation

Input:

- completed LLVM module

Output:

- verified LLVM IR string
- exported symbol manifest

Rule:

- the emitted module must verify before any JIT handoff is allowed

### Stage 6: JIT materialization

Input:

- verified LLVM module

Output:

- executable `JitModule`

Rule:

- symbol lookup happens only after successful verification

## Type Lowering

The initial type mapping should be simple, explicit, and intentionally conservative.

| `IrType` | LLVM type | Notes |
|---|---|---|
| `I8` | `i8` | signedness handled by operations, not storage type |
| `I16` | `i16` | same rule |
| `I32` | `i32` | default integer scalar |
| `I64` | `i64` | 64-bit integer scalar |
| `U8` | `i8` | unsigned semantics in predicates and casts |
| `U16` | `i16` | unsigned semantics in predicates and casts |
| `U32` | `i32` | unsigned semantics in predicates and casts |
| `F32` | `float` | |
| `F64` | `double` | |
| `Bool` | `i1` | branch and compare input |
| `Char` | `i32` | NewCP `CHAR` is a 32-bit Unicode scalar value |
| `ShortChar` | `i8` | |
| `Ptr(T)` | `ptr` | opaque pointer mode |
| `UntaggedPtr(T)` | `ptr` | raw, GC-ignored pointer; same LLVM storage type as `Ptr`, different backend metadata and GC treatment |
| `Ref(T)` | `ptr` | address to storage of `T` |
| `UntaggedRecord { .. }` | concrete LLVM struct or byte blob | no `TypeDesc`; layout controlled by `RecordLayout` |
| `Set(32)` | `i32` | initial subset only |
| `Void` | `void` | function result only |
| `Named(_)` | lowered underlying runtime representation | see below |
| `Opaque(_)` | `ptr` in first slice | placeholder/runtime-owned |

Rules:

- LLVM 22 with Inkwell 0.9 uses opaque pointers, so pointer-bearing IR types should lower to `ptr` and keep pointee meaning in NewCP metadata, not in LLVM pointer types
- signed vs unsigned integers are distinguished by the emitted comparison and division instruction, not by separate LLVM storage types
- `Named` and `Opaque` types should not pretend to have precise structure layouts until the runtime ABI exists; the first slice should lower them conservatively to pointer-shaped values or reject operations that require layout knowledge

### SYSTEM-Driven Type Requirements

`SYSTEM` makes two type-lowering rules non-optional.

- `IrType::UntaggedPtr(T)` lowers to LLVM `ptr` exactly like `Ptr(T)`, but backend metadata must treat it as GC-opaque: no descriptor assumptions, no root registration, no `@Module.Ptrs` entry.
- `IrType::UntaggedRecord { name, layout }` is the first record shape that must lower to a concrete layout before the full tagged-record ABI exists.

Required policy for untagged records:

- `Untagged`: LLVM struct with BlackBox's `MIN(4, size)` field alignment rule
- `UntaggedNoAlign`: packed LLVM struct or explicit byte-offset layout plan
- `UntaggedAlign2`: LLVM struct with `MIN(2, size)` field alignment
- `UntaggedAlign8`: LLVM struct with `MIN(8, size)` field alignment
- `Union`: byte array `[N x i8]` plus field-specific bitcast/GEP views; do not model it as an ordinary non-overlapping struct

No `TypeDesc` global is emitted for an untagged record. Tagged records may remain deferred if their descriptor and header ABI is not yet finalized.

### Named And Opaque Types

The first slice should define two separate policies:

- named scalar-like values that are aliases of builtin types can reuse the builtin LLVM type once sema or IR makes that explicit
- named record, pointer, array, and runtime descriptor shapes lower as opaque pointer-carrying values until the runtime ABI is specified

The important design rule is to avoid smuggling incomplete layout decisions into LLVM structs too early. Once a struct layout leaks into generated IR, it becomes an accidental ABI.

## Storage Model & Garbage Collection Safety

LLVM lowering needs a concrete answer for where mutable state lives, ensuring it is visible to the BlackBox garbage collector. BlackBox uses a precise heap/globals GC and a conservative stack GC.

The initial storage model should be:

- `TempId` values map to LLVM SSA values
- procedure parameters are represented by LLVM function parameters
- local physical allocations (`alloca`) are **safe** because BlackBox sweeps the thread stack conservatively (scanning everything between `SP` and the base stack for aligned pointers); the code generator does not need to emit explicit pointer maps for locals.
- mutable globals **must not** be scattered; they map to field indices inside a single `@Module.Data` struct instance so that `Kernel` has exactly one `varBase` to scan.
- immutable globals map to LLVM global constants when representable, otherwise to read-only globals
- `Ref(T)` parameters are raw addresses passed as `ptr`

Important limitation:

the current IR does not yet distinguish every local variable slot cleanly from globals and symbolic addresses. The backend should therefore start with the subset where addresses already correspond to globals, parameters, or explicit pointer values. If local mutable storage is needed beyond that subset, the correct fix is to add explicit IR storage nodes rather than guessing in the backend.

That means the design should reserve room for a future IR addition such as:

- `IrValue::LocalSlot`
- explicit function-entry `alloca` planning

but the first code generator does not need to invent those concepts internally and hide them from the IR.

## Naming And Symbol ABI

Exported entities need stable names in three spaces:

1. source-level public name
2. LLVM symbol name
3. runtime lookup name

The first slice should use this policy:

- module global variable: `@Module.Name`
- internal helper global: `@__newcp.Module.Name`
- exported procedure: `@Module.Name`
- bound procedure (method): `@ReceiverType_MethodName` (e.g., `@Shape_GetX`, `@Circle_GetX`)
- internal non-exported procedure: `@__newcp.Module.Name`
- runtime trap helper: `@__newcp_trap`

Rules:

- source-exported procedures get public unmangled names of the form `Module.Proc`
- bound procedures use `ReceiverType_MethodName` to avoid name collisions between override chains where two types both declare a method of the same source-level name; this is not module-qualified because the type name already uniquely identifies the scope within a compilation unit
- non-exported procedures get a reserved internal prefix to prevent accidental lookup or collision
- the exported symbol manifest returned by `CompiledModule` should list the public name and exact LLVM symbol name
- globals and procedures should use the same `Module.Name` convention so driver output and future runtime lookup stay aligned

## Procedure Emission Algorithm

Each `IrProcedure` should be lowered with a strict two-pass algorithm.

### Pass A: Declaration And Block Creation

Steps:

1. lower the procedure signature to an LLVM function type
2. create the LLVM function value
3. create one LLVM basic block per `IrProcedure.blocks` entry
4. create an emission context containing:
   - `temp_values: HashMap<TempId, BasicValueEnum>`
   - `block_map: HashMap<BlockId, BasicBlock>`
   - `function: FunctionValue`
   - `result_slot`: optional value only if a later IR revision needs stack storage

Outcome:

- all control-flow targets exist before any terminator is emitted

### Pass B: Instruction And Terminator Emission

Steps:

1. iterate blocks in construction order or RPO, but preserve original CFG identity
2. position the builder at the LLVM block
3. emit each non-terminator instruction in order
4. emit exactly one terminator

Rule:

- every IR block must end with exactly one LLVM terminator, and the emitter must assert that no extra instructions are inserted after the block is closed

## Instruction Lowering Rules

The first implementation should document every `Instr` variant up front.

### `BinOp`

Lowering:

- integer add/sub/mul use integer arithmetic builders
- integer signed divide/mod use signed operations for signed `IrType` and unsigned operations for unsigned `IrType`
- floating add/sub/mul/div use float builders
- comparisons emit `i1`
- `And` and `Or` on booleans use bitwise `and` and `or` on `i1`
- `Shl` and `Shr` are integer shifts; `Shr` must choose logical vs arithmetic based on signedness

First-slice support:

- `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`, `And`, `Or`

Deferred:

- `Xor`, `Shl`, `Shr`, `In` if the IR producer is not yet stable enough to exercise them

### `UnOp`

Lowering:

- `Neg` uses integer negation or float negation based on operand type
- `Not` on booleans is xor-with-true or `not`
- `BitNot` is integer bitwise not

### `Load`

Lowering:

- lower address operand to a pointer value
- emit `load` of the requested LLVM type
- register the loaded SSA value under `dst`

Rule:

- the backend must reject a `Load` when the operand cannot be represented as a pointer-valued LLVM operand

### `Store`

Lowering:

- lower address operand to a pointer value
- lower source operand to a value of the pointee LLVM type
- emit `store`

### `Call`

Lowering:

- resolve callee as either a direct function symbol or an indirect function pointer if supported
- lower arguments in order
- emit call
- if `dst` exists, record the returned SSA value

First-slice rule:

- support direct calls to known LLVM functions first
- reject indirect calls and method-style runtime dispatch until the runtime ABI exists

### `MethodCall`

Status:

- implemented

Emission sequence (see `emit_method_call` in `emit.rs`):

1. Resolve the receiver operand to a `ptr` — this is the object payload address.
2. GEP `obj_ptr - 16` (one `i8` step of -16) to reach the `BlockHeader`.
3. Load the `tag` word (`i64`) from the block header's first field.
4. Mask the low bit: `tag & !1u64` strips the GC mark bit and yields the `TypeDesc` address as an integer.
5. `inttoptr` to `ptr` to get `desc_ptr`.
6. GEP `desc_ptr + 32` (byte offset into `TypeDesc`) to reach the `vtable` field.
7. Load `vtable_ptr` (`ptr`).
8. GEP `vtable_ptr + slot * 8` to reach the correct slot.
9. Load `fn_ptr` (`ptr`).
10. Resolve each argument, prepend the receiver pointer, and emit `build_indirect_call` with a function type inferred from the IR slot and return type.

Key rules:

- the receiver is always passed as the first argument to the target function, before any explicit arguments
- the function type at the indirect call site is built from the IR argument types and `ret_ty`; it must match the signature of the bound procedure as lowered by `newcp-ir`
- slot numbering is fixed at IR-lowering time by `method_slot_in_vtable` and encoded directly in `Instr::MethodCall { slot }` — the backend does not recompute it
- `BlockHeader` is always exactly 16 bytes before the payload pointer; this is part of the allocator/runtime ABI
- byte-GEP (via `i8` element type) is used for the -16 step and the +32 step rather than struct GEP, to avoid requiring the `%TypeDesc` struct type to be declared in every module that dispatches through it

### `AddrOf`

Lowering:

- convert a resolvable symbol into its raw address without loading from it

`SYSTEM` requirement:

- `AddrOf` must lower to `ptrtoint ... to i64`, because NewCP `INTEGER` is 64-bit on the current target
- `AddrOf` of a type symbol means the address of that type's emitted `TypeDesc` global; if the target type is untagged and therefore has no `TypeDesc`, fail with `Unsupported` rather than fabricating one

### `BitCast`

Lowering:

- emit LLVM bitcast, inttoptr, ptrtoint, or integer cast depending on source and destination kinds

Rule:

- not every `BitCast` is a valid LLVM bitcast; the code generator must choose the correct conversion family or reject the IR with `Unsupported`

`SYSTEM` requirement:

- `SYSTEM.VAL` is the main consumer of `BitCast`, so backend lowering must explicitly support `ptrtoint`, `inttoptr`, pointer-domain no-op/bitcast, and same-width integer reinterpretation
- backend assertions should still enforce equal source/destination widths for reinterpreting integer/pointer values even if sema already checked them

### `MemCopy`

Status:

- supported by lowering to `llvm.memmove` when source, destination, and length are representable

Note:

- this is worth adding early because it gives a clear path for `SYSTEM.MOVE`
- use `memmove`, not `memcpy`, because overlap must be correct for BlackBox `MOVE`

### `LoadRaw`

Lowering:

- lower the integer address operand to `i64`
- emit `inttoptr` to a temporary `ptr`
- emit a typed `load` of the requested LLVM value type

Rule:

- this is the direct lowering path for `SYSTEM.GET`; it must not introduce GC metadata or descriptor lookups

### `StoreRaw`

Lowering:

- lower the integer address operand to `i64`
- emit `inttoptr` to a temporary `ptr`
- emit a typed `store`

Rule:

- this is the direct lowering path for `SYSTEM.PUT`; the destination is foreign/raw memory, not a managed heap object

### `Lsh`

Lowering:

- lower both operands to integer SSA values of the source width
- if `shift >= 0`, emit `shl value, shift`
- if `shift < 0`, emit `lshr value, -shift`

Rule:

- this is `SYSTEM.LSH`, not arithmetic right shift; negative shift counts always use logical right shift

### `Rot`

Lowering:

- normalize the shift count modulo the bit width
- lower to `llvm.fshl` / `llvm.fshr` over the source width

Rule:

- prefer LLVM's rotate intrinsics over open-coded shift/or sequences

### `TypTag`

Lowering:

- for managed heap records: compute the header address from the payload pointer, load the tag word, and strip any GC mark bit encoded in-band
- for static stack records: lower to the address of the static `TypeDesc`

Rule:

- if tagged-record header layout is not yet stable, `TypTag` may remain explicitly unsupported rather than guessed

### `SysNew`

Lowering:

- emit `call ptr @__newcp_sys_new(i64 %size)`
- treat the result as an untagged/raw pointer only; no descriptor installation, zeroing, or GC registration is implied

Rule:

- `SysNew` is the LLVM boundary for `SYSTEM.NEW`; it must never lower through `@__newcp_new_rec`

### `TypeCheck`

Status:

- unsupported in the first executable slice unless the operand is lowered through a provisional runtime helper

### `StoreResult`

Lowering:

- in the current IR this is part of the logical-return lowering scheme
- if the IR still uses a synthetic result slot represented as an addressable symbol, lower it as a store to that storage

Design concern:

- once the backend owns physical returns directly from the exit block, this instruction may become unnecessary. The code generator should support the current IR contract without baking it into the long-term ABI.

## Terminator Lowering Rules

### `Br`

- emit unconditional branch to the mapped LLVM block

### `CondBr`

- lower the condition to `i1`
- emit conditional branch to the mapped true and false blocks

### `Ret`

- emit return of the lowered value

### `RetVoid`

- emit `ret void`

### `Trap`

The first slice should use a single runtime helper with a simple ABI:

```text
declare void @__newcp_trap(i32)
```

Lowering:

1. map `TrapKind` to a stable integer code (see table below)
2. emit call to `@__newcp_trap`
3. emit `unreachable`

`TrapKind` to integer code mapping:

| `TrapKind` | Code | Notes |
|---|---|---|
| `Assert` | 1 | ASSERT condition was false |
| `Halt(code)` | `code` | HALT passes its operand directly |
| `NilDeref` | 2 | implicit nil pointer check |
| `ArrayBounds` | 3 | array index out of bounds |
| `TypeGuard` | 4 | WITH/IS guard failed, no ELSE |
| `CaseFallthrough` | 5 | CASE with no matching arm, no ELSE |

These codes are part of the runtime ABI contract. They should be defined once in a shared location (e.g., a `trap_codes` module in `newcp-runtime` or an exported constant block in `newcp-ir`) and imported by both the backend emitter and the runtime trap handler. The design doc records them here as the authoritative definition until that shared location exists.

### `TypeTest`

Status:

- unsupported in the first executable slice unless routed through a provisional runtime helper such as `@__newcp_type_test`

## Value Lowering

`IrValue` should lower by category rather than by syntactic origin.

### Constants

- `ConstInt` -> LLVM integer constant with width from `IrType`
- `ConstReal` -> LLVM float or double constant
- `ConstBool` -> `i1` constant
- `ConstChar` -> `i32` constant for `CHAR` (full Unicode scalar value)
- `ConstStr` -> private global constant array plus `ptr` to its first element. Encoding and termination policy:
  - `String` literals are emitted as null-terminated `[N x i32]` constants (UTF-32; each element is one `CHAR` / Unicode scalar value)
  - `ShortString` literals are emitted as null-terminated `[N x i8]` constants (8-bit bytes; C-compatible)
  - the pointer value exposed to the IR is always a `ptr` (opaque) pointing at element 0
  - the null terminator is included in the constant's length and is always present; the backend must never omit it
- `Null` -> null pointer constant

### Symbolic values

- `Temp` -> lookup in `temp_values`
- `GlobalRef(name, ty)` -> resolution depends on context:
  - if `ty` is a procedure type: lower to the LLVM `FunctionValue` (first-class callable path)
  - if `ty` is any data type: emit a `getelementptr` calculating the exact field offset within the singleton `@Module.Data` struct block (addressable path); callers that need the stored value must emit a `load` themselves
- `ImportRef(module, name, ty)` -> same two-path rule as `GlobalRef`; the symbol was declared as an external declaration during Stage 3 module planning, so lookup goes to that declaration (data imports are currently stubbed but will eventually resolve to a GEP on the imported module's data block)

Rule:

- `AddrOf { sym }` means "yield the raw address of `sym` as an `i64` value" — it is distinct from the addressable path in that it produces an integer SSA value available for `SYSTEM.GET` / `SYSTEM.PUT` / `SYSTEM.MOVE`; it does NOT load the symbol's content
- the emitter needs two helper paths: one that expects a first-class SSA value and one that expects an addressable pointer value; `GlobalRef`/`ImportRef` of a procedure type must only enter the first-class path, and `GlobalRef`/`ImportRef` of a data type must only enter the addressable path; mixing the two paths silently is the most common source of backend type errors

## SYSTEM Backend Requirements

The language-level policy for `SYSTEM` lives in [system-module.md](system-module.md). This section records the LLVM-specific contract once `newcp-ir` already contains explicit SYSTEM-oriented instructions.

Required first-slice SYSTEM support in `newcp-llvm`:

- support `AddrOf`, `BitCast`, `LoadRaw`, `StoreRaw`, `Lsh`, `Rot`, `MemCopy`, and `SysNew` as real codegen paths
- keep `TypTag` specified but allowed to remain `Unsupported` until the tagged-record heap/header ABI is stable
- reject x86-32-only SYSTEM paths with structured `Unsupported` errors if they somehow reach LLVM lowering despite sema gating
- keep all raw addresses in `i64` on the current target
- never synthesize GC-visible metadata for untagged pointers, untagged records, or `SYSTEM.NEW` allocations

Required external declarations when used:

```text
declare ptr @__newcp_sys_new(i64)
declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)
declare iN @llvm.fshl.iN(iN, iN, iN)
declare iN @llvm.fshr.iN(iN, iN, iN)
```

The rotate intrinsic overload depends on the source bit width.

## Dynamic Modular Environment & Linkage Strategy

Unlike static ahead-of-time compilers, `newcp-llvm` generates code for a dynamic, memory-resident environment mirroring BlackBox Oberon. Modules are not just isolated compilation units; they are lifecycle-managed entities that are loaded, linked, executed, and unloaded at runtime.

### Inter-Module ABI and Graph Linkage

When Module `A` calls an exported procedure in Module `B`:
- The compiler emits a direct, strongly-typed LLVM `call` to an external symbol (e.g., `@B.Proc`).
- We do **not** route calls through an indirect Module Descriptor table. The calls are direct and native.
- **Resolution is handled by the JIT linker.** We will structure the JIT environment such that each Component Pascal module becomes its own logical unit in the JIT (e.g., an ORC JIT `JITDylib` context).
- `A` is explicitly told to resolve external symbols by searching the JIT contexts of its imported dependencies (like `B`).

### Module Lifecycle

To support the dynamic paradigm:
1. **Loading:** `newcp-llvm` creates LLVM IR. The runtime backend (ORC JIT) claims it and locks it into memory for execution.
2. **Initialization:** Every module requires a synthesized entry point (e.g., `@A.$init`). The Rust loader uses the JIT to locate this un-mangled symbol and executes it to safely initialize the module's globals.
3. **Replacement & Unloading:** Because inter-module calls are direct, replacing `B` requires strict dependency tracking (the `refcnt` model). If `B` is unloaded, any dependents like `A` must be unloaded first. The JIT will tear down `A`'s memory, then `B`'s, and the new `B` and `A` can be compiled and re-linked seamlessly.

### JIT Engine Choice

Because of this requirement for hot-unloading granular components, the legacy `ExecutionEngine` (MCJIT) is insufficient long-term.
- We must architect `newcp-llvm` to feed an **ORC JIT** backed loader.
- Inkwell provides ORC JIT v2 bindings. We will utilize these to map CP modules 1:1 with `JITDylib`s, enabling safe removal of code when a module is unloaded.

### `JitModule` Responsibilities

The `JitModule` API must abstract the ORC JIT lifecycle:
- encapsulate the module's `JITDylib` or equivalent JIT memory allocation.
- provide `find_exported_function(name)` for external Rust drivers and BlackBox commands.
- provide an explicit `unload()` method that actively removes the dylib from the LLJIT session before any memory is reclaimed. ORC JIT v2 does not reclaim JIT memory automatically on Rust `Drop`; the dylib must be explicitly removed from the session first. Implementing `Drop` as a last-resort call to `unload()` is acceptable but must be documented as a fallback, not the primary path — the loader must call `unload()` explicitly at the right point in the dependency teardown sequence.
- the `JitModule` must refuse to unload if it holds a reference count greater than zero (i.e., other loaded modules still import from it); this check is the loader's responsibility but `JitModule` should expose a query (`ref_count()` or similar) to allow it

### Reflection & Module Descriptors (Future Architecture)

For BlackBox to work, it relies on global data structures describing the module graph and types.
- Eventually, `newcp-llvm` will emit a read-only global constant `Kernel.Module` structure for each module.
- The JIT will expose the address of this descriptor. The Rust kernel will read this address to populate the dynamic module graph, exposing the compiled code via the exact memory layout expected by `Kernel.Mod`.
- For now, we stub out external calls and rely on Rust strings and ORC symbol lookup to find exports.

## Dump And Compile Relationship

`dump-llvm` must not be a parallel code path that manually prints what codegen would have emitted.

Instead:

1. build the real LLVM module
2. verify it
3. print LLVM's own textual IR from that module

That gives one source of truth.

`dump-asm` requires a `TargetMachine`, which `dump-llvm` does not. The rule is:

1. call `compile_ir_module` the same way `dump-llvm` does — there is no separate ASM codepath
2. create a `TargetMachine` from the verified module's target triple, using `CodegenOptions.opt_level` and the host CPU feature set unless a specific triple was provided
3. use the `TargetMachine` to emit textual assembly into a `MemoryBuffer`
4. return the buffer contents as a string

If assembly emission is not yet wired up, `dump_asm` must return `Err(CodegenError::Unsupported { stage: "asm-emission", detail: "TargetMachine assembly output not yet implemented" })` rather than a placeholder string. A placeholder would make the driver appear functional when it is not.

## Unsupported Feature Policy

The backend should classify unsupported cases by design stage.

Examples:

- `Unsupported { stage: "instruction-emission", detail: "MethodCall requires runtime descriptor ABI" }`
- `Unsupported { stage: "value-lowering", detail: "ImportRef for imported variable without declaration planning" }`
- `Unsupported { stage: "type-lowering", detail: "Named record layout not yet specified" }`

This matters because backend bring-up fails in three distinct ways:

- the IR is invalid
- the backend is incomplete
- the runtime ABI is not specified yet

Those should not collapse into one generic error string.

## Test Strategy

The LLVM backend needs tests at three levels.

### 1. Snapshot-style LLVM dump tests

- input: small CP modules
- check: stable substrings in emitted LLVM IR
- include a SYSTEM-focused battery asserting:
	- `SYSTEM.ADR` emits `ptrtoint ... to i64`
	- `SYSTEM.GET` / `SYSTEM.PUT` emit `inttoptr` plus `load` / `store`
	- `SYSTEM.MOVE` emits `llvm.memmove`, not `llvm.memcpy`
	- `SYSTEM.NEW` emits a call to `@__newcp_sys_new`
	- untagged globals do not contribute entries to `@Module.Ptrs`
- include a method-dispatch battery asserting:
	- bound procedures compile with `ReceiverType_MethodName` LLVM names
	- `@TypeName.vtable` constant arrays are emitted with correct slot order (overrides, inherited, new)
	- `@TypeName.desc` constants are emitted with correct `size`, `base`, `vtable`, and `vtable_len` fields
	- derived type descs carry a `ptr @BaseType.desc` in the `base` field

### 2. Verification tests

- input: hand-constructed or lowered `IrModule`
- check: module verifies successfully or fails with the expected `Unsupported`
- include negative cases for:
	- `AddrOf` of a descriptor-less untagged type
	- `TypTag` before the tagged-record ABI is implemented
	- impossible raw-address/type combinations in SYSTEM casts and loads

### 3. JIT smoke tests

- input: exported no-arg procedures
- check: symbol lookup succeeds and the function can be invoked safely

The first tests should avoid over-specifying formatting. They should assert semantic anchors such as:

- exported function exists
- expected branch labels exist
- arithmetic instruction opcode is present
- trap helper declaration is present when traps are used

## Required Near-Term IR Follow-Ups

The design review already surfaces likely IR follow-ups that should be explicit before large backend work starts:

1. add a clearer distinction between globals, imported symbols, locals, and explicit stack slots
2. decide whether `StoreResult` remains part of the IR contract or becomes a purely lowering-internal mechanism
3. decide how function symbols and data symbols are distinguished in `IrValue`
4. decide how runtime-assisted operations like type tests are represented at the IR boundary (method dispatch is now resolved — see `Instr::MethodCall` above)

These are not blockers for the first executable subset if the subset is narrow. They are blockers for pretending the whole backend is ready.

## Staged Implementation Plan

The implementation should proceed in these milestones.

### Milestone 1: Real LLVM module plumbing

- add `inkwell = { version = "0.9.0", features = ["llvm22-1"] }`
- switch `dump_llvm` from parser-based placeholder rendering to `newcp-ir`-based lowering
- initialize native target and create/verify a module
- emit empty/void procedures with correct basic blocks and returns

Exit criteria:

- `dump-llvm` shows real LLVM IR from LLVM itself
- modules with empty procedures verify successfully

### Milestone 2: Scalar executable subset

- add constants, temps, arithmetic, loads, stores, branches, and returns
- add exported symbol manifest
- add simple JIT wrapper for exported no-arg procedures
- include the P1 SYSTEM paths that do not depend on tagged-record descriptors: `AddrOf`, `BitCast`, `LoadRaw`, `StoreRaw`, `Lsh`, `MemCopy`, `SysNew`

Exit criteria:

- a test module can be compiled, looked up by exported name, and executed through the JIT

### Milestone 3: Runtime helper integration

- add trap helper calls
- add provisional runtime hooks for operations that require runtime knowledge
- replace placeholder unsupported cases only when the helper ABI is defined
- add `@__newcp_sys_new` declaration and call emission as a required runtime hook
- add descriptor/tag-aware lowering for `TypTag` only after the tagged heap/header ABI is stable

Exit criteria:

- trap-bearing CFG lowers to verified LLVM with explicit runtime helper calls

### Milestone 4: Method dispatch

- slot numbering in IR lowerer (`method_slot_in_vtable`, `collect_type_vtables`)
- bound procedure naming convention (`ReceiverType_MethodName`) and receiver-as-first-param lowering
- `@TypeName.vtable` and `@TypeName.desc` constant global emission in `declare_type_descs`
- `Instr::MethodCall` backend emission in `emit_method_call`

Status: **complete** (commit be69bd0).

### Milestone 5: ABI expansion

- parameters beyond the trivial subset
- imported symbol resolution
- memory intrinsics and selected casts
- more complete integer and floating operations
- untagged record layout emission (`align8`, `align2`, `noalign`, `union`) and exclusion from GC root metadata

Exit criteria:

- the backend covers the intentionally chosen bootstrap language subset rather than isolated demo procedures

## Final Design Position For The First Slice

The first code generator should be intentionally modest but fully real.

It should:

- compile from `newcp-ir`, not from parsed source summaries
- emit real verified LLVM IR via Inkwell
- expose that exact LLVM IR through `dump-llvm`
- JIT exported procedures from a narrow scalar subset
- fail explicitly for IR operations that still depend on unresolved runtime ABI work

It should not:

- guess missing IR storage semantics inside the backend
- invent record or descriptor layouts before the runtime ABI is designed
- keep placeholder `dump-llvm` and real codegen as separate paths
- treat JIT success on one demo function as proof that the backend architecture is complete

That is the boundary between a structured backend bring-up and a backend that becomes impossible to reason about six weeks later.