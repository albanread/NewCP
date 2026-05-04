# Roadmap

## Phase 0: Design freeze for the first slice

Deliverables:

- language subset definition
- runtime ABI sketch
- module registry design
- descriptor layout sketch
- 64-bit field-width policy for descriptors, fixups, and runtime metadata
- Rust-resident `Kernel` and `Init` bootstrap sketch
- decision that runtime/bootstrap does not depend on a `Sym/` folder
- LLVM ORC integration choice
- phase dump format definitions
- explicit minimal bootstrap rule: only build the Rust-resident services and temporary facade modules required to self-host and replace them with CP modules

Exit criteria:

- enough detail to begin coding without reopening the architecture each day
- no unresolved ambiguity about which runtime fields are pointer-sized versus compact metadata
- clear distinction between live in-memory interface metadata and any future persisted interface cache
- clear statement of which modules are required before self-hosting and which are deferred until after CP replacements begin

## Phase 1: Front-end bootstrap

Deliverables:

- lexer
- parser
- AST model
- diagnostics
- test corpus for syntax
- token dump format
- AST dump format

Exit criteria:

- parse `MODULE ... END` files reliably
- recover from syntax errors well enough for tests
- produce stable textual token and AST dumps

## Phase 2: Semantic core

Deliverables:

- binder
- type checker
- import resolver
- export table builder
- semantic dump format
- first explicit in-memory interface model for exported symbols and types

Exit criteria:

- type-check small multi-module programs
- build symbol tables for exported objects and types
- produce stable textual semantic dumps

## Phase 3: IR and CFG

Deliverables:

- CFG builder
- typed IR
- procedure lowering
- CFG dump format
- IR dump format

Exit criteria:

- lower nontrivial control flow and calls without AST-dependent codegen
- produce stable textual CFG and IR dumps

## Phase 4: LLVM backend and JIT

Deliverables:

- LLVM IR generator
- runtime intrinsic bridge
- ORC JIT session
- module materialization pipeline
- LLVM IR dump support
- assembly dump support

Implemented so far (as of commit be69bd0):

- real LLVM IR emission from `newcp-ir` via Inkwell (no placeholder)
- scalar arithmetic, loads, stores, branches, calls, returns
- `SYSTEM` intrinsics: `AddrOf`, `BitCast`, `LoadRaw`, `StoreRaw`, `Lsh`, `Rot`, `MemCopy`, `SysNew`
- trap helper calls
- type tests (`IS`/`WITH`) via `Instr::TypeCheck` + `Terminator::TypeTest`
- tagged record allocation via `Instr::New` / `@__newcp_new_rec`
- inherited field access via typed GEP across the extension chain
- **method dispatch**: vtable and TypeDesc constant globals, `Instr::MethodCall` indirect dispatch, bound-procedure naming and slot numbering

Exit criteria:

- JIT one module and call one exported command
- emit reviewable LLVM IR and assembly for that module

## Phase 5: Runtime compatibility shell

Deliverables:

- Rust `Kernel` equivalent
- Rust `Init` equivalent
- module registry
- type registry
- export/import linker
- basic reflection lookups
- module initialization order
- minimal temporary Rust facade modules only where bootstrap is blocked without them

Exit criteria:

- load module A importing B, initialize B then A, invoke an exported command from A
- start from the Rust bootstrap, then compile and register the first CP modules into the live process
- replace at least one temporary Rust-hosted module with its CP-built equivalent without changing the surrounding bootstrap model

## Phase 6: Memory-resident BlackBox shell

Deliverables:

- persistent in-memory module residency
- late module loading
- safe service registration
- basic trap cleanup integration

Exit criteria:

- a long-lived process can JIT more modules on demand without restarting

## Phase 7: Framework recovery

Deliverables:

- command model
- reflection support expected by tools
- enough runtime support to begin porting or rehosting core BlackBox modules
- optional persisted interface/symbol cache if tooling and rebuild workflows justify it

Exit criteria:

- core framework modules can be compiled and loaded under NewCP conventions

## Immediate next implementation steps

1. create the `src` and `tests` layout under `NewCP`
2. lock Rust as the implementation language
3. scaffold the front-end projects
4. define the first typed IR data model
5. define the resident runtime interfaces for module registration and symbol lookup
6. define the textual dump contracts for each phase
7. write down the first concrete 64-bit layouts for module descriptors, type descriptors, and relocation-bearing metadata
8. define the Rust `Kernel` and `Init` responsibilities before adding more CP-side modules
9. define the in-memory interface descriptor model before deciding whether any persisted symbol cache is worth adding
10. keep the bootstrap module set minimal and justify every additional Rust-hosted module against the self-hosting path

## Recommendation on host language

Use Rust for the host compiler and runtime.

Reasons:

- native deployment without a managed VM
- one language for runtime, compiler, and driver
- suitable for systems work and compiler implementation
- practical interop with LLVM through bindings or a thin C bridge

Keep LLVM behind a narrow backend boundary so front-end work remains independent of binding choice and code generation strategy.