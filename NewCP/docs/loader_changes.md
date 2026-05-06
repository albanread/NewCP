# NewCP Loader vs. Legacy BlackBox Loader

This document outlines the architectural differences and the philosophical shift between the original BlackBox Component Builder dynamic loader (`StdLoader`) and the new NewCP `LoaderSession` (`newcp-loader`).

## The "No Unload" Philosophy

The most significant ideological shift in the new loader revolves around continuous execution and quiescent unloading. The NewCP loader is designed to continually recompile the code of a "running" application on the fly without the user worrying about manual operations, while ensuring memory safety during hot swaps.

In the BlackBox environment, memory management relies on a Garbage Collector rather than explicit `dispose` operations. We extended this philosophy to module lifecycle management. Just as a developer shouldn't manually dispose of memory, they shouldn't need an explicit `unload` operation to hot-swap code. The `LoaderSession` handles dirty state tracking, dependency invalidation, and seamlessly re-materializes the sub-graph on the fly without halting the host process. Memory is reclaimed lazily once execution scopes bound to the archaic code conclude via the quiescence model.

## Architectural Comparison

### 1. Scope: Dynamic Linker vs. Orchestrator
*   **BlackBox (`StdLoader`):** Acted purely as a dynamic binary linker. It ingested pre-compiled `.ocf` (Object Code Files) created by the Component Pascal compiler, read byte headers, requested memory blocks, and manually performed relocation pointer assignments. 
*   **NewCP (`LoaderSession`):** Acts as an end-to-end continuous execution orchestrator. Because compilation and execution are fused, NewCP takes `.cp` source files natively, orchestrates parsing (`newcp-parser`), semantic integrity validation (`newcp-sema`), LLVM IR generation (`newcp-llvm`), and handles the LLVM ORC JIT linkage on the fly. 

### 2. Error Tracking and Coherency
*   **BlackBox:** Handled errors via hardcoded integers to represent states like `fileNotFound`, `cyclicImport`, or `illegalFPrint`. If a module load sequence failed halfway, it could leave the environment's module registry tainted or unable to instantiate.
*   **NewCP:** Features granular operations natively inside the API. `LoaderFailurePhase` explicitly isolates errors out of `DiscoverGraph`, `ReadModuleSource`, `Parse`, `Analyze`, `Codegen`, or `Materialize` steps. Explicit `invalidation_state` and fallback caching (`CachedSourceGraph`) provides fault-tolerant edit-fix-retry loops that strictly track intermediate graphs and automatically roll back partial load failures to maintain coherency.

### 3. Versioning and Fingerprints (The Hot-Reload Shift)
*   **BlackBox:** Handled module versioning using strict interface fingerprints embedded as `fp` binary segments inside `.ocf` files. Modifying an upstream module would cause downstream loads to hard-fault with an `illegalFPrint` error, requiring manual recompilation of dependents.
*   **NewCP:** Operates on live file boundaries and source dependency graph topology instead of arbitrary binary fingerprints. Using `SourceFileStamp` (combining file size and UNIX modification time timestamps), NewCP evaluates the `dirty_modules`, automatically traces the dependents, flags `RetirementReason::DependencyChanged`, and regenerates them to fulfill the dependency graph context dynamically.

### 4. Code Generation & Jump Addresses
*   **BlackBox:** The loader's `Fixup()` method implemented handwritten x86 jump displacement replacements (e.g., `absolute`, `relative`, `deref`, `table`) to knit function calls together in live memory space, binding it strictly to 32-bit architectural constraints.
*   **NewCP:** Utterly bypasses handwritten pointer fixups by subordinating linking procedures directly to modern compiler infrastructure. By generating an `OwnedJitModule` backed seamlessly by the LLVM execution engine's layout rules natively through Rust wrappers, cross-architecture, fully-optimized code with dynamic symbol mapping is guaranteed. Sub-modules that are JIT compiled export functions to an actively merged session module map, resolving exports inherently.

### 5. Generational Scoping and Execution Pins
*   **BlackBox:** Code execution ran linearly natively in shared address space. Replacing a loaded module meant manual unloading and unlinking of running systems risking crashes.
*   **NewCP:** Implements a concept of Epochs and Generations tracked via `ExecutionScopeId`. `PinnedGeneration` structs dictate precisely what version of a module is under act of execution by a specific command or operation. Overwriting a source file triggers generation updates (`generation = n+1`) allowing the active task (`n`) to safely wind down naturally, before collecting retired variants automatically during the next quiescent epoch.

## Conclusion
The NewCP loader completely supersedes the legacy `StdLoader` operations. We've transitioned from manually decoupling application execution and module linking to an automated, self-healing system capable of hot-swapping code at runtime—true to the "safety and simplicity" ethos of Component Pascal while backing it with LLVM level optimization and modern generational lifetimes.