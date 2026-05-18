# NewCP — a personal recreation of Component Pascal and the BlackBox environment

This does not work, may take years, and has an incompetent gc. But we will see.

This repository is a from-scratch recreation of the Component Pascal language and the BlackBox Component Builder environment, implemented on a modern toolchain (Rust + LLVM) and aimed at modern hardware (64-bit, JIT-first, multi-gigabyte address spaces).

The work in this tree is a personal project. I always liked Oberon/F. I find Component Pascal one of the easiest languages to read and to understand, and I want to recreate it — but for the machines that exist now, not the ones it was originally squeezed onto.

## A short history of the lineage

### Oberon (1986–1992)

Niklaus Wirth and Jürg Gutknecht designed the **Oberon language and operating system** at ETH Zürich, building on Wirth's earlier Pascal and Modula-2 work. The design intent was radical for its time: a complete operating system, compiler, editor, document model, and GUI written in a single small language, fitting comfortably on a workstation that today would be considered a microcontroller. The Oberon system pioneered:

- garbage collection in a systems language
- text-as-active-medium: every word in the system is a clickable command
- a tiling, non-overlapping window model where text frames *are* the application surface
- type extension (single inheritance) baked into the language

The companion book — *Project Oberon: The Design of an Operating System and Compiler* — described the entire system end-to-end, hardware up.

### Oberon-2 (1991)

Hanspeter Mössenböck and Wirth added **type-bound procedures** (methods) and read-only export to Oberon, producing Oberon-2. This was the first version of the Oberon family with conventional object-orientation; it kept everything else minimal.

### Oberon/F → BlackBox Component Builder (1994–2014)

Cuno Pfister and a small team at **Oberon microsystems** in Zürich — a spin-off from ETH founded around 1993 — took Oberon-2 and built a commercial development environment around it. They first called it **Oberon/F** ("F" for Framework). It was a properly engineered MVC component framework with a serializable document model (`Stores`), live document composition (any view could embed any other view, recursively), and a forms / text engine that doubled as the layout engine.

Around 1997 Oberon microsystems renamed Oberon/F to **BlackBox Component Builder** and, in the same release, refined the Oberon-2 language slightly — tightening some object-oriented mechanics, formalizing the component contract — and rebranded the result as **Component Pascal**. Component Pascal is essentially Oberon-2 with cleaner OO semantics and a stronger module/component story.

BlackBox shipped commercially through the late 1990s and 2000s, was used in real applications (notably in industrial control, scientific computing, and embedded tooling), and was eventually open-sourced. Oberon microsystems also produced the Bluebottle / Active Oberon line and a JVM-targeted variant (Component Pascal for JVM), but BlackBox on Windows is the one most people remember.

What made BlackBox unusual:

- a full development environment — compiler, debugger, inspector, profiler, source editor — that ran inside the same memory space as the user's application, as ordinary modules
- a document-centric model (`Stores`, `Models`, `Views`, `Controllers`) where every screen artifact was serializable to an `.odc` rich-document file
- direct-to-COM on Windows: COM objects were first-class, with reference counting tied into the GC
- dynamic module loading and unloading at the language level, with type-safe cross-module references
- the entire framework in maybe 60,000 lines of Component Pascal — small enough to read end-to-end in a few weekends

### Why I'm recreating it

I always liked Oberon/F.

Component Pascal is, to my eye, one of the easiest languages to read and to understand. The grammar is small, the semantics are obvious, the standard library is concrete, and the framework is straightforward MVC without ceremony. Reading a Component Pascal module feels closer to reading a careful technical specification than reading code in most modern languages.

But the original BlackBox was a child of its hardware era: 32-bit, megabytes of RAM, an x86 backend hand-written in Component Pascal, an object-file loader inherited from a world where memory was scarce and compilation was something you did once and persisted to disk.

My modern hardware is different. I have 64-bit pointers, gigabytes of RAM, fast SSDs, and an LLVM toolchain that will produce better x86_64 than I will ever write by hand. The whole BlackBox source tree fits in a fraction of a single page of physical memory. There is no reason the framework cannot live entirely resident, no reason modules cannot be JIT-compiled on first use, no reason a 64-bit address space should not be the default.

So **NewCP** ([NewCP/README.md](NewCP/README.md)) is the recreation: the same language, the same framework shape, the same MVC discipline, the same `.odc` document model — but built on:

- Rust for the resident runtime, compiler, garbage collector, and JIT loader
- LLVM (via Inkwell, MCJIT today, ORC v2 later) for code generation
- Direct2D + DirectWrite for the integrated GUI (`iGui`), replacing the old `wingui.dll` host
- 64-bit-first descriptors, fixups, and runtime metadata
- a phase-visible compiler pipeline where every stage emits stable textual dumps suitable for review and regression testing
- the `HostXxxSys` layering pattern: abstract `Xxx.cp` modules stay import-free, `HostXxx.cp` provides the host implementation, and only the `Sys` shim imports the GUI / OS layer

The goal is not literal binary compatibility with BlackBox 1.7. The goal is to recreate the *programming model* — Component Pascal modules, dynamic loading, the Stores/Views/Models/Controllers framework, the document-centric world — in a system that takes modern hardware as a given.

## Repository layout

```
NewCP/        the language, compiler, runtime, framework, and GUI
YAML/         supporting tooling
review-md/    review tooling
review-md-test/
tools/        miscellaneous developer tools
```

See [NewCP/README.md](NewCP/README.md) for the current status of the language, compiler, runtime, framework slice, and integrated GUI; for the test counts; and for the rolling next-milestones plan.

## Influences and credit

This project owes everything to the prior work of Niklaus Wirth, Jürg Gutknecht, Hanspeter Mössenböck, Cuno Pfister, and the Oberon microsystems team. NewCP is not affiliated with ETH Zürich or Oberon microsystems; it is a personal recreation built from the public BlackBox documentation, the open-sourced BlackBox 1.7 sources, the Oberon-2 / Component Pascal language reports, and a long-standing affection for Oberon/F.

The original BlackBox documentation, source releases, and the *Project Oberon* book remain the authoritative references for the language and the system being recreated.
