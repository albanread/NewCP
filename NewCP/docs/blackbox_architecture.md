# BlackBox Component Builder Architecture

This document summarizes the high-level architecture and design philosophy of the original BlackBox Component Builder, based on its official documentation and source layout. The goal is to provide context on the legacy system that NewCP is evolving from.

## Core Philosophy

BlackBox is a complete, self-contained development and application environment built around **Component Pascal** (a refinement of Oberon-2). It goes beyond being just a compiler; it acts as a minimalist operating system and UI framework running on top of the host OS (typically Windows).

Key architectural tenets:
- **Dynamic Loading and Linking:** Applications are not monolithic executables. The system is composed of independently compiled modules that are loaded, linked, and garbage-collected on demand by the `Kernel`.
- **Component-Oriented:** Rather than building closed applications, developers build extensible components. A component might be a new text view, a GUI control, a database connector, or a completely new application extending existing boundaries.
- **Direct-To-COM (DTC):** On Windows, the compiler and runtime provide native, seamless interop with Microsoft's Component Object Model (COM), handling reference counting automatically by tying it into the language's garbage collector.
- **Safety by Default:** Strongly typed, bounds-checked, and garbage-collected.

## Subsystem Structure

The BlackBox architecture is strictly organized into hierarchical *subsystems*. Within each subsystem, code is divided into `Mod` (source code), `Docu` (documentation), `Rsrc` (resources like strings and dialog forms), and `Sym`/`Code` (binary outputs).

The primary subsystems are:

###  1. The Core Infrastructure
- **`System`:** The absolute core of BlackBox. It contains the Module loader, Garbage Collector (`Kernel`), reflection/meta-programming definitions (`Meta`), base abstractions for the Component Framework (`Views`, `Models`, `Controllers`, `Stores`, `Ports`, `Fonts`), and standard libraries (`Strings`, `Math`, `Files`).
- **`Host`:** The platform adaptation layer. It binds abstract `System` interfaces to the concrete host operating system (e.g., implementing `HostFiles` over Win32 file APIs). Note that BlackBox historically targeted Windows almost exclusively, with experimental Linux/Mac versions.
- **`Std`:** Standard, platform-independent implementations of `System` interfaces. This includes standard dialogs, generic containers, and basic tools.

### 2. The Model-View-Controller Framework
BlackBox operates a deeply integrated GUI using a pure MVC pattern:
- **Models (`System.Models`):** Manage application data and state.
- **Views (`System.Views`):** Graphical representations of models. Anything drawn on screen is a View. Views exist in a hierarchy; the text subsystem itself is just a container of Views.
- **Controllers (`System.Controllers`):** Manage user interaction (mouse, keyboard) and route messages to models and views.
- **Stores (`System.Stores`):** The serialization layer. Everything on screen can be serialized to a `.odc` document via the `Stores` architecture, providing persistence out of the box.

### 3. Application Subsystems
- **`Text`:** The incredibly powerful rich-text engine. Because text is just a specialized `View` containing other `Views`, it functions as the UI layout engine. All documentation, source code, and logs in BlackBox are rich text `.odc` files capable of hosting live GUI controls.
- **`Form`:** A forms subsystem for data entry screens and traditional GUI building, tightly integrated with `Controls`.
- **`Win` / `Ole` / `Com` / `Ctl`:** Direct abstractions over Windows APIs, OLE embedding, and COM bindings.
- **`Comm`:** Networking and serial communications.
- **`Sql`:** Database integration (via ODBC/OLEDB typically). 
- **`Xhtml`:** Basic HTML generation and parsing capabilities.

### 4. The Development Environment (`Dev`)
The compiler and development tooling are just regular modules running inside the framework.
- **`Compiler` (`DevCP*`)**: The Component Pascal compiler pipeline.
  - `CPS` (Scanner/Lexer)
  - `CPP` (Parser)
  - `CPT` (Abstract Syntax Tree)
  - `CPM` (Module/Symbol Table)
  - `CPH/CPB/CPE/CPC486/CPL486/CPV486` (x86 Code Generation)
- **`Linker` / `Packer`**: Tools for producing standalone `.exe` files when the dynamic module loader is not desired.
- **`Debug`, `Inspector`, `Analyzer`**: Deeply integrated development and profiling tools. Because code runs within the same memory space as the IDE, debugging involves inspecting live `Meta` object graphs directly.

### 5. The "Action" Mechanism
BlackBox relies on late-bound *Commands* rather than standard entry points (like `main()`).
Any exported parameter-less procedure can be executed dynamically. The UI triggers these commands (e.g., a button press runs `MyModule.DoThing`). This decoupling is what allows the framework to reload modules and manage an event loop internally.

## Conclusion for NewCP
The NewCP project brings Component Pascal into the modern era with LLVM, removing the tight coupling to Microsoft COM and x86 32-bit architecture. However, understanding exactly why `Module` -> `Store` -> `View` -> `Controller` was designed this way helps interpret the design decisions found in existing CP source code, where modules expect a resident, highly interactive runtime environment.

## NewCP vs BlackBox: Source-First Open Architecture

NewCP is philosophically aligned with BlackBox's component model but differs from it in one fundamental respect: **NewCP is entirely source-driven and open source**.

### What NewCP inherits from BlackBox conceptually

- The component model: modules are independently loaded, versioned, and replaceable at runtime.
- The command model: any exported parameterless procedure is a first-class command.
- Dynamic module loading and hot-swapping without stopping the process.
- Garbage-collected heap with precise object metadata (`TypeDesc`).
- Subsystem organisation: `Mod/` mirrors BlackBox's per-subsystem `Mod/` folder convention. `Mod/Tests/` holds test modules; other subsystem folders (e.g. `Mod/BlackBox/`, `Mod/Std/`) can be added to group related modules.

### Where NewCP fundamentally diverges

| Dimension | BlackBox | NewCP |
|---|---|---|
| Distribution | Binary `.ocf` / `.sym` files | **Source `.cp` files only** |
| Compilation | IDE-internal, produces binaries | **JIT from source at load time** |
| Interface contract | Binary `.sym` symbol files | **Source `DEFINITION MODULE` or live sema** |
| Architecture | 32-bit x86 Windows | **64-bit, LLVM-backed, cross-platform** |
| Licence | Proprietary (Oberon microsystems / ETH) | **Open source** |
| Bootstrap | Requires pre-built BlackBox environment | **Builds itself from Rust + CP source** |

### The key consequence: no binary artefacts in the repo

NewCP does not ship, load, or link any pre-compiled `.ocf`, `.sym`, or object files. Every module — whether part of the standard library, a subsystem, or an application — is distributed as a `.cp` source file and compiled on demand by the NewCP JIT pipeline.

This means:

- the entire system is auditable at source level
- the compiler and the runtime are updated together with no ABI slip
- adding a new module is as simple as dropping a `.cp` file into the right `Mod/` subfolder
- the loader's recursive `Mod/` search means subsystem folders become first-class citizens with no configuration needed
