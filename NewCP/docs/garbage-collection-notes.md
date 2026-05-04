# BlackBox Garbage Collection: Architecture & Codegen Implications

## How the BlackBox GC Works

Through analyzing `review-md/System/Mod/Kernel.odc.md`, we mapped exactly how BlackBox's garbage collector discovers and marks memory. It uses a **hybrid** approach: **conservative on the stack**, but **precise on the heap and globals**.

### 1. Conservative Stack Scanning
When a GC cycle begins, the stack is scanned between the current Stack Pointer (`SP`) and the original `baseStack`. The GC reads every word on the stack. If the word's value falls inside the bounds of a managed heap cluster and is properly aligned (16-byte boundaries), it assumes it is a valid pointer and marks the block it points to.
- **Implication:** The code generator does *not* need to maintain precise stack maps. Local variables, parameters, and compiler-generated temporaries living on the stack or in registers will be found automatically. LLVM's standard local variable spilling is perfectly safe.

### 2. Precise Heap Objects
Memory dynamically allocated via `NEW()` is prefixed with a `tag`. 
- `tag` points to a `Kernel.Type` descriptor (`TypeDesc`).
- The `Type` descriptor contains an array called `ptroffs`, containing the byte offsets of every pointer within that memory block.
- When an object is marked, the GC adds it to the list and walks `ptroffs` to find child pointers.
- **Implication:** To allocate memory, `newcp-llvm` must not simply call `malloc` or `llvm.allocate`. It must lower `NEW()` to the BlackBox runtime call `Kernel.NewRec(typeTag)`. The type descriptors must eventually be synthesized in LLVM globals so their addresses can be passed to `NewRec`.

### 3. Precise Module Globals
During the mark phase, `Kernel` iterates through a linked list of loaded modules (`modList`).
- For each module `m`, it pulls `m.varBase` (the beginning address of the module's global mutable data) and `m.ptrs` (an array of offsets).
- It iterates `i = 0..m.nofptrs` and reads `varBase + ptrs[i]`, treating any non-zero value there as a managed heap pointer.
- **Implication:** Module globals cannot be emitted as scattered standalone LLVM globals (e.g. `@Module.myVar1`, `@Module.myVar2`). They *must* be packed into a single, contiguous LLVM `struct` definition (`@Module.Data`), with a single global instance. Then, the compiler must crawl that struct's layout to compute the byte offset of every pointer it contains, building an array of integers to populate `ptrs`.

## Impact on LLVM Codegen Design

The realization that BlackBox requires a single `varBase` for module globals completely invalidates using scattered LLVM `GlobalVariable` objects for source globals. We must update the code generation design strictly.

1. **GlobalPlanner Revisions:** Stage 3 must aggregate all source-level variables into a single global struct (e.g., `%ModuleDataStruct`).
2. **Access Lowering:** Accessing a global variable becomes a `getelementptr` on the `@Module.Data` singleton, rather than directly referencing a dedicated `@MyVar` LLVM global. 
3. **Pointer Offset Table:** The `GlobalPlanner` must compute the struct layouts using the data layout rules and generate a synthesized LLVM constant array holding the `varBase` pointers offset list, ensuring the GC will be able to sweep the module when it is eventually linked into `Kernel.modList`.
4. **Stack Confidence:** We explicitly document that `newcp-llvm` will rely on BlackBox's conservative stack scanner, meaning we can use LLVM's `alloca` freely without worrying about stack roots.