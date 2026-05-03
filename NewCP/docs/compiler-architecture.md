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
- constant folding where unambiguous
- export surface construction

Output:

- typed AST or HIR
- module symbol table
- runtime descriptor plan for exported types and procedures

Observable artifacts:

- bound tree dump
- symbol table dump
- exported surface dump
- type descriptor plan dump

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

Each procedure body lowers to a control-flow graph.

Responsibilities:

- basic block construction
- explicit branches for `IF`, `CASE`, loops, `RETURN`, `EXIT`
- explicit exception/trap edges where needed later
- data-flow-ready form for optimization and codegen

This is the right place to normalize the language before LLVM lowering.

Observable artifact:

- per-procedure CFG dump with named basic blocks, terminators, and edges

### Typed IR

NewCP should own a small typed IR instead of lowering directly from AST to LLVM.

Reasons:

- easier language-specific diagnostics
- simpler runtime-specific lowering
- easier descriptor and metadata generation
- freedom to target other backends later

Recommended properties:

- explicit loads/stores
- typed operations
- explicit call nodes
- explicit runtime intrinsics
- explicit module/global references
- CFG-based procedures

Observable artifact:

- stable textual IR dump suitable for tests and manual review

## Backend

### LLVM IR generation

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

The first vertical slice should be:

1. parse a small module with one global and two procedures
2. dump tokens and AST
3. type-check it
4. dump semantic state
5. build CFG and typed IR
6. dump CFG and IR
7. emit LLVM IR
8. dump LLVM IR and assembly
9. JIT it
10. register exports in runtime metadata
11. invoke an exported command through the runtime

The second slice should add imports and module initialization.

After that, the next slice should prove that the live Rust-hosted system can compile and materialize at least one nontrivial Component Pascal support module into the already-running environment.