# Compiler Architecture

## Intent

Build NewCP the usual modern way, but target a live module system rather than only a static executable pipeline.

Bootstrap constraint:

- keep the resident Rust core minimal
- add only the modules and services required to start the compiler, load the first CP modules, and then replace temporary Rust facades with CP-built modules
- postpone broader framework recovery until after that bootstrap replacement loop works end to end

The pipeline is:

1. lexer
2. parser
3. AST
4. semantic binder and type checker
5. module graph and import resolver
6. control-flow graph lowering
7. typed mid-level IR
8. LLVM IR generation
9. ORC JIT materialization
10. runtime module registration

## Execution target

The initial NewCP compiler and runtime target is a 64-bit JIT environment.

Baseline assumptions:

- pointer width is 64 bits
- module, heap, descriptor, and code addresses are 64-bit values
- the first supported host/target pair is `x86_64-pc-windows-msvc`
- 32-bit compatibility is explicitly out of scope for the first implementation

This should affect ABI planning from the start rather than being treated as a later porting detail.

It should also affect scope control: the first implementation is allowed to be narrow as long as it is sufficient to bootstrap the compiler and hand responsibility to CP modules as early as practical.

It also implies a field-class policy:

- address-like fields are 64-bit by default
- descriptor references and symbol addresses are 64-bit by default
- offsets and compact metadata fields must be justified explicitly if they remain narrower
- legacy 32-bit structure layouts are references for semantics, not templates for exact in-memory width

## Phase contract

Every major compiler phase must be discrete and observable.

Discrete means:

- each phase has a clear input model and output model
- each phase can be tested independently
- each phase can fail with diagnostics before later phases run
- phase boundaries are explicit in the codebase rather than hidden inside one pass

Observable means:

- each phase has a stable textual representation for review
- the driver can stop after any major phase and emit that representation
- regression tests can compare normalized textual output

This requirement applies at least to:

- tokens
- parse tree or AST
- bound symbols and type information
- module dependency graph
- CFG
- typed IR
- LLVM IR
- final assembly

The compiler should therefore support commands equivalent to `dump-tokens`, `dump-ast`, `dump-sema`, `dump-module-graph`, `dump-cfg`, `dump-ir`, `dump-llvm`, and `dump-asm`.

## Front end

### Lexer

Responsibilities:

- Latin-1 source compatibility where required
- Unicode in strings as needed
- comments and nested comments
- keywords and identifiers
- numeric literal decoding including hex forms
- precise source spans for diagnostics

Output:

- token stream with trivia and source spans

Observable artifact:

- normalized token dump with token kind, lexeme, and source span

### Parser

Responsibilities:

- full Component Pascal grammar
- error recovery good enough for tooling
- module-oriented parsing, not single-file fragments only

Output:

- syntax tree for module, declarations, statements, and expressions

Observable artifact:

- stable textual parse tree or AST dump

### Semantic analysis

Responsibilities:

- scope/binding
- imported module resolution
- type formation and identity
- extension hierarchy validation
- method rules and override checks
- parameter passing mode validation
- constant folding for all literal types (integer, real, char, string, boolean)
- export surface construction

**Constant values:** the `ConstValue` enum carries typed constant results — `Integer(i128)`, `Real(f64)`, `Char(char)`, `String(String)`, `Boolean(bool)`. Every `SemanticSymbol` of kind `Constant` has a `const_value` and an inferred `declared_type`.

**Builtin scope:** every module analysis begins with a preloaded set of builtin types (`INTEGER`, `REAL`, `BOOLEAN`, `CHAR`, `BYTE`, etc.), builtin procedures (`INC`, `DEC`, `NEW`, `ASSERT`, `ABS`, `ODD`, etc.), and builtin constants (`TRUE`, `FALSE`, `INF`). These are injected at the start of `module_symbols` before any module declarations are processed.

**Selector resolution:** ambiguous parse nodes (`AmbiguousParen`) are resolved during sema. A single bare-identifier argument `P(Name)` is resolved as a call if `P` is a known procedure, or as a type guard if `Name` is a known type. The resolution is recorded in `SelectorResolution` for each procedure and for module-level statements.

**WITH guards:** sema validates that the guarded variable is a record-typed VAR parameter or receiver (including imported Named types), and that the guard target names a record type that extends the static type of the variable. Resolutions are recorded as `TypeGuard` entries for use by CFG lowering.

**CASE labels:** integer constants, character constants (including single-char string literals `"a"`), and named constants are valid CASE labels. Char range labels (`"a".."z"`) are supported. Duplicate and out-of-order ranges are rejected.

Output:

- `SemanticModule` with `symbols`, `procedures`, `selector_resolutions`, `diagnostics`
- per-procedure `SemanticProcedure` with `signature`, `local_symbols`, `selector_resolutions`, `diagnostics`

Observable artifacts:

- `dump-sema`: structured per-symbol and per-procedure dump including symbol kinds, types, constant values, selector resolutions, and diagnostics

## Middle end

### Module graph

The compiler must reason about modules as a dependency graph, not as isolated compilation units.

Responsibilities:

- discover imports
- detect cycles
- define initialization order
- define JIT materialization dependencies

Observable artifact:

- module graph dump with dependency edges and initialization order

### CFG lowering

Each procedure body lowers from the typed `SemanticProcedure` to a control-flow graph of basic blocks.

The CFG is built directly from the typed AST — there is no flat TAC pass first. Component Pascal's structured control flow maps directly to CFG shape without needing to first flatten structure that was never lost.

**Basic block definition:**

A `BasicBlock` has an ID, a list of typed instructions, and exactly one terminator. No fall-through between blocks — every block ends with an explicit terminator. This keeps the CFG unambiguous and makes predecessor/successor lists trivially correct.

**Control flow lowering rules:**

| CP construct | CFG shape |
|---|---|
| Sequential statements | single block |
| `IF … ELSIF … ELSE … END` | chain of condition blocks, each branching to body or next test; all arms merge at a join block |
| `CASE` (integer, char) | decision tree of comparisons (dense ranges: jump table candidate later); each arm block merges at join |
| `WHILE cond DO … END` | head block tests condition → body block → back-edge to head; else → exit block |
| `REPEAT … UNTIL cond` | body block → test block → back-edge to body or → exit block |
| `LOOP … END` | body block → back-edge to body; `EXIT` → explicit branch to loop-exit block |
| `EXIT` | `Br` to the nearest enclosing loop's exit block (maintained on a loop stack during lowering) |
| `RETURN` | `Ret` terminator in the logical IR; lowered to `store result_slot, v` + `Br function_exit`; the function-exit block ends with the single physical `ret load result_slot` |
| `WITH v: T DO … END` | `TypeTest` instruction → conditional branch to guard body; refined type annotation on the guarded variable inside that block |

**`EXIT` and `LOOP` implementation:** the lowering pass maintains a stack of `(loop_continue_target, loop_exit_target)` block IDs. `EXIT` pops the top and emits `Br loop_exit_target`. `LOOP` pushes a new pair before lowering the body. The `Br` emitted by `EXIT` should carry the current loop stack depth as debug metadata — this is invaluable when diagnosing "wrong loop exited" bugs, particularly when `CASE`, `WITH`, or nested `LOOP` statements are involved and the stack depth at the exit site is not obvious from a linear reading of the IR.

**`RETURN` and the function-exit block:** lowering maintains two distinct return forms. The logical IR uses `Ret(v)` / `RetVoid` as emitted by the lowering of `RETURN` statements — these are the instructions that appear in the instruction set definition. Internally, each `Ret(v)` is lowered to:

```
store result_slot, v
Br function_exit
```

The single `function_exit` block then ends with the one physical `ret load result_slot`. This keeps the CFG uniform (every block has exactly one terminator; no block has multiple successors via return), collects all epilogue work — stack cleanup, debug info, ABI return convention — in one place, and produces consistent debug info line mappings across all return paths. For void procedures the result slot is omitted and `function_exit` ends with `RetVoid`. The distinction between logical `Ret` (lowering input) and physical `Br function_exit` (lowering output) should be explicit in the IR representation rather than left implicit.

Responsibilities:

- basic block construction
- explicit branch terminators for all control flow
- loop stack for `EXIT` resolution
- `RETURN` collection at a function-exit block
- `TypeTest` instruction for `WITH`/`IS` guards
- data-flow-ready form for later optimization

Observable artifact:

- per-procedure CFG dump with named basic blocks, instructions, terminators, and edges
- Graphviz `.dot` output for visual review during bring-up
- block ordering annotation on every block: both its construction index (the order blocks were created during lowering) and its RPO index (reverse post-order over the finished CFG)

Construction index and RPO index are almost always different, and that difference is where bugs hide. A back-edge to a loop head is obvious in RPO order and invisible if you only see construction order. Conversely, a block that was constructed early but ends up unreachable is obvious in construction order and disappears from RPO. Printing both indices on every block in the textual dump — e.g. `bb3 [c=3 rpo=1]` — costs nothing and removes an entire class of "why is this block here?" debugging sessions.

### Typed IR

NewCP owns a small typed IR (`newcp-ir`). The IR is not a separate pass after CFG construction — the CFG **is** the IR. The `IrProcedure` contains basic blocks populated with typed instructions.

**Instruction set:**

Values and data movement:
```
t = Const(v, ty)
t = Load(addr, ty)
Store(addr, v)
t = BinOp(op, a, b, ty)        -- add/sub/mul/div/mod/and/or/xor/shl/shr
t = UnOp(op, a, ty)            -- neg/not/bitnot
t = Call(f, args, ret_ty)
t = MethodCall(descriptor, slot, args, ret_ty)
t = AddrOf(sym, ty)            -- SYSTEM.ADR lowering
t = BitCast(v, from_ty, to_ty) -- SYSTEM.VAL lowering
MemCopy(dst, src, len)         -- SYSTEM.MOVE lowering
```

Control flow (terminators):
```
Br(target)
CondBr(cond, true_target, false_target)
Ret(v) / RetVoid
Trap(code)                     -- ASSERT, HALT, array bounds, nil check
TypeTest(v, ty, true_target, false_target)
```

**Type system:**

```
enum IrType {
    I8, I16, I32, I64,
    U8, U16, U32,        -- BYTE, SHORTINT unsigned forms
    F32, F64,            -- SHORTREAL, REAL
    Bool,
    Char, ShortChar,
    Ptr(Box<IrType>),    -- explicit pointer to T
    Ref(Box<IrType>),    -- VAR parameter reference
    Named(String),       -- opaque imported or forward type (source-level name)
    Opaque(String),      -- runtime-internal types: descriptors, vtables, tag words
    Set(u8),             -- CP SET type; width in bits (32 for SET, extensible later)
    Void,
}
```

`Named` and `Opaque` are intentionally distinct. `Named` refers to a type that has a source-level identity in a Component Pascal module — the name can be resolved against the module graph. `Opaque` refers to runtime-internal structures (type descriptor headers, vtable arrays, module anchor records) that have no CP source definition and should never be exposed to language-level type checking. Keeping them separate prevents the lowering pass from accidentally treating a vtable pointer as a user type.

`Set(width)` is included from the start because CP sets are not integers — they have distinct operations (`INCL`, `EXCL`, `IN`, `*`, `+`, `-`) that lower to bitwise ops but carry set semantics through the IR. Representing them as a plain integer would silently drop that information before codegen has a chance to use it. Width 32 covers the standard `SET` type; the field exists to accommodate `LONGSET` or similar extensions without an IR change.

Typed IR values carry their `IrType` so that LLVM IR generation is a straightforward structural mapping without needing to re-infer types.

**Lowering entry point:**

```rust
fn lower_procedure(proc: &SemanticProcedure, module_symbols: &[SemanticSymbol]) -> IrProcedure
```

Sema diagnostics have already been emitted — this pass trusts the types and only lowers. The first working target is a procedure with no control flow; add `IF`, then `WHILE`/`REPEAT`, then `LOOP`/`EXIT`, then `CASE`, then `WITH`.

Reasons for owning this IR:

- language-specific diagnostics (e.g. definite assignment, unreachable code) are expressed cleanly at this level
- runtime-specific lowering (descriptors, tags, `NEW`, set operations) lives here, not in LLVM IR generation
- descriptor and metadata generation can walk the IR independently of LLVM
- freedom to target other backends (e.g. a bytecode interpreter for bootstrap or testing)

Observable artifact:

- stable textual IR dump suitable for tests and manual review
- one block per line, instructions indented, terminators marked explicitly

## Backend

### LLVM IR generation

We have elected to use the `inkwell` crate for LLVM bindings. It provides a safe, strongly-typed Rust wrapper over the LLVM C API, which significantly reduces the likelihood of segfaults and mismatched types during code generation.

LLVM IR generation is responsible for:

- target data layout for a 64-bit process
- data layout mapping
- procedure code generation
- globals for module data
- descriptor objects for runtime metadata
- indirect call lowering for procedures and methods
- runtime helper calls for allocation, type tests, strings, sets, and traps

LLVM is a backend, not the source of truth for language semantics.

Observable artifact:

- full LLVM IR dump per module

### ORC JIT

Use LLVM ORC JIT for module loading.

Responsibilities:

- symbol interning by `Module.Symbol`
- materialization units per Component Pascal module
- lazy compilation on first load or first use
- relocation/resolution against runtime symbols and already-loaded modules
- stable function/global addresses after registration

The unit of JIT loading is a Component Pascal module, not an individual procedure.

Observable artifacts:

- module materialization log
- resolved symbol map
- final assembly dump when requested

## Runtime-facing code generation rules

### Module emission

Each compiled module must produce:

- code for procedures and module body
- storage for globals
- export directory metadata
- type descriptors
- signature descriptors
- import table metadata
- pointer maps for runtime scanning

For the bootstrap slice, implement only the subset of these structures that is required for the first replacement-capable modules. Do not widen the emitted surface merely to mirror the eventual full runtime contract before the compiler can use it.

Those structures must be specified for a 64-bit process from the outset, especially for addresses, descriptor references, and relocation-bearing metadata.

### Method dispatch

Record-bound procedures should lower to:

- descriptor-backed dispatch slots
- runtime-stable method numbering
- direct or indirect LLVM calls depending on static knowledge

### Allocation

Generated code should call runtime intrinsics for:

- `NEW` record allocation
- dynamic array allocation
- string/runtime helpers as needed

Do not let frontend codegen bypass the runtime allocator.

## Runtime architecture

The runtime is split into three layers.

### 1. Resident core

Always live in memory:

- `Kernel` equivalent
- `Init` bootstrap support
- module registry
- type registry
- symbol resolver
- heap / GC interface
- trap and cleanup support
- command dispatcher
- ORC JIT session

This layer is implemented in Rust.

The compiler and driver are also resident Rust components. They are not themselves the first Component Pascal modules; they are the machinery that lexes, checks, lowers, JITs, and registers those modules.

### 2. Base modules

JITed early and typically kept resident:

- reflection/meta services
- files/services infrastructure
- minimum text/document services later

These are the first Component Pascal modules expected to be compiled into memory by the resident Rust-hosted compiler/runtime.

### 3. Demand-loaded modules

JITed when imported or explicitly requested.

Examples:

- tools
- converters
- document families
- UI subsystems

These modules are also compiled into memory by the same running compiler/JIT pipeline rather than prelinked into the resident host.

## Bootstrap shape

The first executable system should follow this order:

1. start a Rust-resident `Kernel` equivalent
2. start a Rust-resident `Init` equivalent
3. initialize the resident compiler pipeline inside the live process
4. bring up the runtime registries, allocator/GC boundary, and ORC JIT session
5. compile and JIT the first Component Pascal base modules into memory
6. register those modules and execute their module bodies in dependency order
7. continue by compiling or loading more modules on demand

The compiler must be available before the first CP modules are materialized, otherwise the bootstrap path becomes circular.

The boundary should be explicit from the start:

- the resident Rust compiler emits a compiled-module artifact
- the runtime kernel registers that artifact as a live module
- normal BlackBox-like import resolution and module-body initialization then apply

For the early system, the runtime may also host a small number of Rust facade modules that present the same module-and-command surface as future CP modules. Those facades are transitional: they should be replaceable by later compiled modules under the same module name.

This keeps the bootstrap small, explicit, and compatible with the long-term goal of a memory-resident environment that grows by compiling more modules into itself.

## Non-goals for the first implementation

- full binary compatibility with legacy `.ocf`
- exact memory layout parity for every old implementation detail
- eager support for all BlackBox document/view families
- aggressive optimization
- true unload/reclaim of arbitrary live modules

## First executable slice

Steps and current status:

| # | Step | Status |
|---|---|---|
| 1 | parse a module; dump tokens and AST | done — `dump-tokens`, `dump-ast` commands working |
| 2 | type-check it; dump semantic state | done — `dump-sema` command; all 7+ test modules clean |
| 3 | driver `check-mod` / `check-dir` commands | done — exit 1 on any diagnostic |
| 4 | build CFG and typed IR; dump both | next — `newcp-ir` crate is stubbed, needs real lowering |
| 5 | emit LLVM IR; dump it | pending `newcp-llvm` |
| 6 | JIT it via ORC | pending |
| 7 | register exports in runtime metadata | pending |
| 8 | invoke an exported command through the runtime | pending |

The second slice adds imports and module initialization order.

After that, the next slice proves that the live Rust-hosted system can compile and materialize at least one nontrivial Component Pascal support module into the already-running environment.

**Test modules in `Mod/` (all pass `check-dir`):**

| Module | Constructs covered |
|---|---|
| `Empty.cp` | minimal module, no declarations |
| `Consts.cp` | exported constants: integer, real, string, char, boolean |
| `Vars.cp` | basic-type vars, export marks, BEGIN init block |
| `Procs.cp` | IF/ELSIF, FOR, WHILE, INC/DEC builtins |
| `Records.cp` | RECORD types, VAR params, field access, SIMD shapes |
| `Pointers.cp` | POINTER TO RECORD, NEW, ASSERT, pointer field access |
| `TypeExt.cp` | RECORD extension, inherited field assignment |
| `Loops.cp` | REPEAT/UNTIL, LOOP/EXIT |
| `CaseWith.cp` | CASE on integer and char, WITH type-guard dispatch (cross-module) |