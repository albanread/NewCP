# BlackBox JIT Compatibility

## Goal

Recreate the BlackBox execution model closely enough that a Component Pascal environment can come alive inside one process and load more modules dynamically at runtime.

Implementation rule for the first slice:

- keep the resident Rust compatibility shell as small as possible
- add only the hosted modules needed to bootstrap the compiler and load the first CP replacements
- treat every Rust-hosted framework module as provisional unless the compiler still depends on it

The compatibility target is a 64-bit recreation of that model, not a 32-bit clone of the legacy deployment environment.

The compatibility target is behavioral, not binary. NewCP does not need to reproduce the legacy `.ocf` loader byte-for-byte in its first version. It does need to preserve the runtime contracts that BlackBox code expects.

## What the extracted sources already tell us

### Language

The language report identifies the essential runtime-facing features:

- separate compilation
- dynamic loading of modules
- garbage collection
- type extension and method dispatch
- module initialization via module bodies

### Runtime module model

The `Kernel.Module` descriptor exposes the module-level ABI shape:

- module name
- options and flags
- reference count
- terminator command
- import list
- code/data/reference regions
- exported object directory
- names table and pointer tables

This is the anchor for the JIT loader design.

### Runtime type model

The `Kernel.Type` descriptor exposes the type ABI shape:

- type size
- owning module
- encoded type id/form/attributes
- base type vector
- field directory
- pointer offsets
- method table slots at negative offsets for records

This is the anchor for dynamic type tests, method dispatch, reflection, and GC scanning.

### Reflection and late binding

`Meta`, `Services`, `Documents`, `Views`, `Files`, `Converters`, and related modules rely on:

- enumerating loaded modules
- looking up modules and types by name
- looking up exported commands and procedures
- keeping descriptors alive while modules are loaded
- refusing actions on unloaded modules via `refcnt < 0`

### Legacy object and symbol formats

The extracted object and symbol format documents show the old system's durable ABI concepts:

- symbol files carry signatures, type structure, fields, methods, visibility, and import/export identity
- object files carry module blocks, type descriptors, code/data sizes, fixups, imported uses, and entry points

NewCP should preserve these semantics internally even if the initial implementation does not emit `.ocf` and `.osf` files.

Design decision:

- the live runtime does not require a `Sym/` folder or on-disk symbol files to load modules
- the in-memory module/type/interface descriptors are the primary source of truth for loading, reflection, and compatibility checks inside the running system
- a persisted symbol or interface cache may be added later for incremental builds, tooling, or offline compatibility checks
- if such a cache is added, it should be derived from the in-memory interface model rather than forcing the runtime to depend on a `Sym/` directory at bootstrap time

## Required compatibility contract for NewCP

### 1. Module lifecycle

NewCP runtime must support:

- `load by module name`
- import dependency resolution
- single initialization of each loaded module
- ordered execution of imported modules before importer body execution
- persistent module registry
- optional module invalidation/unload state

Compatibility rule: a module body is the module initializer and must execute exactly once per live module instance.

### 2. Export/import ABI

A module must expose enough metadata to resolve:

- exported constants
- exported types
- exported variables
- exported procedures/commands
- type descriptors and signatures

NewCP runtime must provide a name-based module export directory comparable to `Kernel.Directory`.

For the first memory-resident implementation, import resolution should work directly against resident module/interface metadata. Persisted symbol artifacts are optional tooling support, not a prerequisite for module loading.

### 3. Type ABI

Every runtime type must have a stable descriptor containing:

- owning module identity
- type form and attributes
- size / shape information
- base-type links
- field metadata
- pointer map for GC/scanning
- procedure signature metadata for procedure types
- dispatch layout for record-bound procedures

Compatibility rule: dynamic type tests and method dispatch must be based on runtime descriptors, not only compile-time assumptions.

### 3a. ABI sizing policy

Because NewCP is 64-bit first, ABI sizing must be specified explicitly instead of inherited accidentally from the 32-bit system.

Required policy:

- pointers, code addresses, data addresses, and descriptor references are 64-bit values in the initial runtime
- module registry entries, export directory entries, imported symbol bindings, and JIT symbol addresses must be designed around 64-bit residency
- any field that names an address-like location must be treated as 64-bit unless there is a specific bounded reason not to
- byte offsets, table indexes, and compact encoded metadata may remain narrower, but only when they are deliberately specified as non-address values
- legacy `Kernel.Module` and `Kernel.Type` layouts should be treated as semantic references, not as literal field-width templates

Compatibility rule: do not preserve a 32-bit field width merely because the legacy structure used one. Preserve meaning first, then choose a width suitable for a 64-bit resident runtime.

### 4. Command ABI

BlackBox makes heavy use of exported parameterless procedures as commands. NewCP must support:

- exported commands discoverable by module/name
- late invocation by string path, e.g. `Module.Command`
- command registration without static relinking

### 5. Reflection ABI

The runtime must support:

- `ThisMod(name)`
- `ThisType(module, name)`
- module scanning
- field/method/signature enumeration
- module residency checks

Without this, much of the BlackBox framework stops being dynamic.

This reflection model is also the basis for any future persisted symbol/interface cache. The runtime should not maintain one representation for live use and a separate unrelated one for tooling.

### 6. Memory model

The original system is GC-based and descriptor-driven. NewCP must preserve:

- heap allocation for records and arrays
- pointer-aware tracing/scanning
- correct handling of dynamic arrays and record extension
- safe interaction between JITed code and the managed heap
- 64-bit-safe descriptor, pointer, and code-address handling throughout the runtime

This affects concrete runtime structures such as:

- module descriptors
- type descriptors
- export/import binding tables
- pointer maps and GC metadata
- JIT symbol tables
- fixup or relocation records used inside the runtime pipeline

The design assumption is conservative memory residency:

- keep module code and descriptors resident
- keep type descriptors resident for the lifetime of the process by default
- delay real unloading until GC, trap cleanup, and callback safety are fully specified

## Explicit architectural change

NewCP changes one major assumption of the original system:

The environment is memory-resident first.

That means:

- the runtime starts as a small resident core
- it can JIT itself into a usable environment in-memory
- it JITs more modules as they are imported or requested
- it is acceptable to keep compiled modules, descriptors, metadata, and service registries live
- the system optimizes for simplicity and dynamism, not disk-era memory frugality

This is compatible with the BlackBox programming model even though it is more aggressive about residency.

It also means the runtime ABI should be specified directly in 64-bit terms from the start. If a future 32-bit port is ever desired, it should be treated as a separate portability effort rather than an implicit constraint on the first design.

## LLVM consequence

LLVM is not the compatibility target. The runtime contract above is the compatibility target.

LLVM ORC JIT is the implementation mechanism for:

- compiling modules lazily
- materializing code/data for a module
- resolving imported symbols against already-loaded modules and runtime intrinsics
- preserving live code pointers while the process remains active

## First compatibility subset

The initial JIT-compatible subset should support:

- modules
- imports
- constants, variables, procedures
- records, pointers, arrays, dynamic arrays
- type-bound procedures
- exported commands
- module initialization
- descriptor-backed reflection for exported symbols

That is enough to get a usable runtime shell before tackling full document/view/store compatibility.

It is also enough to guide scope: do not implement broader BlackBox subsystem compatibility during bootstrap unless it is directly required to get from resident Rust services to CP-compiled replacement modules.