# Component Pascal Language And Compiler Notes

## Purpose

This note captures two things needed for NewCP:

- the basic formal definition of Component Pascal from the official language report
- the structure of the existing BlackBox compiler implementation modules

The goal is not to copy the old compiler architecture. The goal is to understand the language and the existing implementation well enough to design a cleaner modern compiler while preserving compatibility where it matters.

## Where the formal language definition lives

Primary language reference:

- [../../review-md/Docu/CP-Lang.odc.md](../../review-md/Docu/CP-Lang.odc.md)

Related runtime and ABI references:

- [../../review-md/System/Mod/Kernel.odc.md](../../review-md/System/Mod/Kernel.odc.md)
- [../../review-md/System/Mod/Meta.odc.md](../../review-md/System/Mod/Meta.odc.md)
- [../../review-md/System/Mod/Services.odc.md](../../review-md/System/Mod/Services.odc.md)
- [../../review-md/System/Mod/Stores.odc.md](../../review-md/System/Mod/Stores.odc.md)
- [../../review-md/Dev/Spec/SymFile.odc.md](../../review-md/Dev/Spec/SymFile.odc.md)
- [../../review-md/Dev/Spec/ObjFile.odc.md](../../review-md/Dev/Spec/ObjFile.odc.md)

## Basic Component Pascal facts from the official report

### High-level character

Component Pascal is not Pascal in the classic sense.

It is closer to Oberon-2 with a component-oriented runtime model and stronger emphasis on:

- separate compilation
- strong static typing across module boundaries
- extensible record types
- methods bound to record types
- dynamic loading of modules
- garbage collection

The language report explicitly presents Component Pascal as a refinement of Oberon-2, not as a continuation of Pascal.

### Lexical and source basics

From the language report:

- source syntax is specified in EBNF
- base source representation is ISO 8859-1 / Latin-1
- Unicode is allowed in string literals
- identifiers are case-sensitive
- comments are nested `(* ... *)`
- reserved words are upper-case keywords

### Core declaration model

The language is block-structured and module-oriented.

Core declaration categories:

- constants
- types
- variables
- procedures
- modules

Export marks are part of the language surface:

- `*` means exported
- `-` means read-only or implement-only, depending on the declaration kind

This export model matters directly for symbol files, metadata, runtime reflection, and the module ABI.

### Type system basics

The official report defines:

- scalar basic types such as `BOOLEAN`, `SHORTCHAR`, `CHAR`, `BYTE`, `SHORTINT`, `INTEGER`, `LONGINT`, `SHORTREAL`, `REAL`, `SET`
- fixed arrays and open arrays
- records with attributes such as `ABSTRACT`, `EXTENSIBLE`, `LIMITED`
- pointers to records and arrays
- procedure types
- string values as null-terminated character arrays

Important non-Pascal properties:

- records can be extended
- pointer types inherit extension relationships from their base record types
- `ANYREC` and `ANYPTR` exist as runtime-facing top types
- dynamic type matters for method dispatch and guards

### Object model basics

The language report describes an object model based on extensible records and methods.

Important features:

- methods belong to record types
- method dispatch uses dynamic type
- type guards refine the static type inside a designator
- record and pointer variables have static and dynamic type concepts

This is a major difference from Pascal and a core reason NewCP needs runtime descriptors from the beginning.

### Procedure and module basics

The report defines:

- procedures with formal parameter modes
- methods as type-bound procedures
- module bodies as executable initialization logic
- dynamic module loading as part of the language environment contract
- finalization support

This means the compiler cannot stop at syntax and type checking. It must participate in a module system.

## Why Component Pascal differs from Pascal

At the level relevant to NewCP, Component Pascal differs from classic Pascal in several foundational ways:

- module system rather than program/unit style rooted in static linking
- dynamic loading as a normal environment feature
- garbage-collected heap model
- record extension and dynamic dispatch
- runtime type tests and guards
- stronger separation between exported and non-exported module surface
- reflection and runtime metadata expectations in the BlackBox system

If NewCP is treated as "Pascal with modules", the design will be wrong.

## Why Component Pascal is not just Oberon-2 either

The official report treats Component Pascal as a refinement of Oberon-2, but the BlackBox environment adds practical runtime expectations beyond the language core:

- richer module metadata and reflection
- persistent module/type descriptors
- command discovery and invocation
- document/view/store integration
- symbol and object file contracts that support dynamic loading and tooling

For NewCP, compatibility means implementing both the language and the BlackBox runtime expectations layered on top of it.

## Existing BlackBox compiler implementation

The compiler implementation is modular, but it is not separated into clean modern phases.

The most important implementation modules visible in this workspace are:

### Compiler entry and orchestration

- [../../review-md/Dev/Mod/Compiler.odc.md](../../review-md/Dev/Mod/Compiler.odc.md)

Role:

- user-facing compiler commands
- orchestration of the internal compiler modules
- setup of options and logging
- calls into parser/front-end, metadata/export, and backend

Observation:

- this is a command package and coordinator, not the compiler core itself

### Shared compiler state and core data model

- [../../review-md/Dev/Mod/CPT.odc.md](../../review-md/Dev/Mod/CPT.odc.md)

Role:

- core compiler object model
- names, constants, objects, structures, nodes
- symbol/type/node data structures
- global compiler state shared across modules

Observation:

- this plays the role of shared AST/symbol/type infrastructure, but in a very global mutable style

### Scanner / tokenizer

- [../../review-md/Dev/Mod/CPS.odc.md](../../review-md/Dev/Mod/CPS.odc.md)

Role:

- lexical scanning
- token classification
- string and identifier decoding
- numeric literal handling

Observation:

- there is a real scanner module, so the compiler is modular at the source level

### Parser with semantic construction mixed in

- [../../review-md/Dev/Mod/CPP.odc.md](../../review-md/Dev/Mod/CPP.odc.md)

Role:

- parsing expressions, declarations, procedures, blocks, modules
- building compiler nodes
- performing semantic work during parsing

Observation:

- this is the clearest sign the old compiler is not phase-separated in a modern sense
- parsing and semantic construction are strongly interleaved

### Semantic helpers / compatibility checks / tree building utilities

- [../../review-md/Dev/Mod/CPB.odc.md](../../review-md/Dev/Mod/CPB.odc.md)

Role:

- type compatibility
- parameter checking
- assignment checking
- expression compatibility
- tree helper functions

Observation:

- this contains logic a modern compiler would usually split between binder, type checker, and IR builder support

### Compiler manager / object and symbol file support / diagnostics

- [../../review-md/Dev/Mod/CPM.odc.md](../../review-md/Dev/Mod/CPM.odc.md)

Role:

- compiler options and diagnostics
- object and symbol file handling
- constants for backend/object model
- compiler-wide services

Observation:

- this mixes environment concerns, file-format concerns, and compiler control concerns

### Export and metadata emission

- [../../review-md/Dev/Mod/CPE.odc.md](../../review-md/Dev/Mod/CPE.odc.md)

Role:

- metadata/object emission support
- exported object/type representation
- descriptor and fixup-related constants

Observation:

- this is closer to a metadata/object writer than a pure backend

### Machine-specific backend

- [../../review-md/Dev/Mod/CPV486.odc.md](../../review-md/Dev/Mod/CPV486.odc.md)
- [../../review-md/Dev/Mod/CPL486.odc.md](../../review-md/Dev/Mod/CPL486.odc.md)
- [../../review-md/Dev/Mod/CPC486.odc.md](../../review-md/Dev/Mod/CPC486.odc.md)

Role:

- i386-specific code generation
- register and instruction-level backend support
- lowering from compiler nodes/items into machine-oriented form

Observation:

- the backend is target-specific and tightly coupled to the old node/object model

## Assessment of the old compiler architecture

### What is useful

The old compiler is useful as a compatibility oracle for:

- accepted language details
- semantic edge cases
- symbol and object metadata meanings
- module/export rules
- backend assumptions that shaped runtime ABI

### What is not a good model for NewCP

It is not a good direct model for NewCP phase architecture because:

- parser and semantics are intertwined
- global mutable state is shared across modules
- backend assumptions leak into front-end structures
- machine-specific details are close to the core representation
- phase boundaries are not designed for inspection or textual dumps

This matches the expectation that it is modular, but not phased like a modern compiler.

## NewCP conclusion

NewCP should keep the old compiler as a semantic and compatibility reference, not as an architectural template.

The right strategy is:

- keep formal language basics grounded in the official report
- use the BlackBox compiler modules to discover implementation realities and edge cases
- design NewCP with explicit, observable phases: lexer, parser, sema, module graph, CFG, typed IR, LLVM IR, JIT/runtime registration
- provide textual dumps for every phase so the new architecture is reviewable in a way the old one is not

## Immediate follow-up references

For future language-compatibility work, the next modules worth mining are:

- [../../review-md/Dev/Mod/Analyzer.odc.md](../../review-md/Dev/Mod/Analyzer.odc.md)
- [../../review-md/Dev/Mod/Browser.odc.md](../../review-md/Dev/Mod/Browser.odc.md)
- [../../review-md/Dev/Mod/Linker.odc.md](../../review-md/Dev/Mod/Linker.odc.md)
- [../../review-md/Dev/Mod/Dependencies.odc.md](../../review-md/Dev/Mod/Dependencies.odc.md)

These are likely to expose additional accepted-language behavior, metadata assumptions, and module-graph expectations.