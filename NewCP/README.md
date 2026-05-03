# NewCP

NewCP is a new Component Pascal compiler and runtime intended to recreate the BlackBox programming model with a modern in-memory architecture.

The current engineering rule is to keep the bootstrap slice minimal: implement only the resident Rust pieces and temporary facade modules required to bring up the compiler, compile the first CP modules, and then replace those temporary Rust modules with CP-built equivalents.

The target is not a batch compiler that emits native object files as its primary product. The target is a memory-resident system that can:

- parse and type-check Component Pascal modules
- lower them through a modern compiler pipeline
- generate LLVM IR
- emit reviewable textual artifacts from every compiler phase
- JIT modules into memory
- load additional modules dynamically on demand
- preserve enough of the BlackBox runtime model to support documents, commands, views, stores, reflection, and dynamic services

## Core decision

NewCP is JIT-first.

NewCP is also 64-bit first.

The original BlackBox loader worked with object files, symbol files, type descriptors, fixups, and dynamic module initialization. NewCP keeps the dynamic module model, but the default execution model is different:

- the runtime is memory-resident
- the initial implementation targets a 64-bit address space
- the first supported execution target is `x86_64-pc-windows-msvc`
- module code, data, descriptors, and metadata stay resident by default
- modules are JIT-compiled when first required
- additional modules may be JIT-compiled later into the same process
- unloading is optional and secondary; correctness of dynamic loading matters more than memory reclamation

Memory is no longer treated as the scarce resource that shaped the original loader. BlackBox is small enough now that the whole environment can live in memory.

This should be read as a deliberate break from the original 32-bit deployment assumptions. A 32-bit runtime is not a design target for the first system.

## Design set

- [docs/blackbox-jit-compatibility.md](docs/blackbox-jit-compatibility.md)
- [docs/compiler-architecture.md](docs/compiler-architecture.md)
- [docs/roadmap.md](docs/roadmap.md)

## Phase visibility

NewCP must not behave like an opaque compiler pipeline.

Each major phase should be:

- discrete in implementation
- invokable independently where practical
- observable through stable textual dumps
- suitable for regression tests and human review

The expected review artifacts include:

- token stream dumps
- parse tree / AST dumps
- bound symbol and type dumps
- module graph dumps
- control-flow graph dumps
- typed IR dumps
- LLVM IR dumps
- final assembly dumps

This is a design requirement, not an optional debugging feature.

## Host language note

Rust is the current preferred implementation language for both the resident runtime and the compiler.

Reasons:

- native deployment without a managed VM
- suitable for low-level runtime and kernel work
- suitable for compiler front-end and IR work
- good fit for keeping the whole system in one toolchain
- practical interop options with LLVM through bindings or a narrow native bridge

In concrete terms, the pieces expected to be written in Rust from the start are:

- the resident `Kernel` equivalent
- the startup/bootstrap `Init` equivalent
- the compiler pipeline and driver
- the JIT loader and runtime symbol/link infrastructure

Component Pascal modules are then compiled by that Rust-hosted system and materialized into memory on demand.

Anything beyond that should be treated as deferred unless it is strictly required to bootstrap the compiler or to replace an existing Rust-hosted facade with a CP module.

## Application shape

NewCP should begin with the same broad split that BlackBox used:

- a resident `Kernel` equivalent for module tables, type descriptors, runtime services, and loader/JIT coordination
- a separate `Init` equivalent that boots the application, loads the first modules, registers core services, and enters the live environment

This is a better starting point than treating application startup as an undifferentiated blob. The compiler itself should live alongside that resident core as a Rust component and should compile Component Pascal modules into the running process.

## Source layout target

This folder is currently design-first. The intended implementation layout is:

```text
NewCP/
  docs/
  Mod/
  src/
    newcp-lexer/
    newcp-parser/
    newcp-sema/
    newcp-ir/
    newcp-llvm/
    newcp-runtime/
    newcp-loader/
    newcp-driver/
  tests/
    newcp-tests/
    newcp-compat-tests/
```

## First build target

The first useful end-to-end target is:

1. lex and parse a module
2. dump tokens and syntax
3. build typed IR
4. lower procedures into CFG form
5. dump CFG and IR
6. emit LLVM IR
7. dump LLVM IR and final assembly
8. JIT a module body
9. register the module in a runtime module table
10. call exported commands from the runtime

That is the smallest slice that proves the architecture.

For now, "smallest" should be read aggressively: if a service, module, descriptor surface, or compatibility feature is not required to get from Rust bootstrap to the first self-hosting CP replacements, it should stay out of the implementation slice.

## Current scaffold

The Rust workspace has been created with these crates:

- `newcp-lexer`
- `newcp-parser`
- `newcp-sema`
- `newcp-ir`
- `newcp-llvm`
- `newcp-runtime`
- `newcp-loader`
- `newcp-driver`
- `newcp-tests`
- `newcp-compat-tests`

There is also a top-level `Mod/` folder for Component Pascal source modules that the resident compiler can compile and load.

The driver surface is intentionally phase-oriented from the start and is expected to grow around commands such as:

- `describe-interface`
- `load-module`
- `dump-tokens`
- `dump-ast`
- `dump-sema`
- `dump-module-graph`
- `dump-cfg`
- `dump-ir`
- `dump-llvm`
- `dump-asm`

Current convention:

- `newcp-driver describe-interface InitShell` shows the in-memory interface descriptor for a live module
- `newcp-driver load-module HostMenus` looks for `Mod/HostMenus.cp`
- `newcp-driver load-module HostMenus HostMenus.OpenApp` loads the module and invokes a command immediately

## Minimal bootstrap set

The current target is not to recreate the whole BlackBox framework up front. The target is to keep just enough live infrastructure in Rust to compile and replace modules incrementally.

The intended minimal bootstrap set is:

- resident `Kernel`
- resident `Init`
- resident compiler pipeline and driver
- resident JIT loader and runtime registries
- only the smallest temporary Rust facade modules needed to let the compiler start and load CP modules
- a `Mod/` source folder containing the first CP modules we expect to compile into the running process

The intended replacement rule is:

- start with Rust only where the compiler cannot yet provide the module
- once a CP module can be compiled and loaded reliably, prefer the CP module over the Rust facade
- do not add new Rust-hosted framework modules unless the bootstrap path is blocked without them