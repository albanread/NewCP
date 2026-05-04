# NewCP Garbage Collector Design

## Overview

The Component Pascal runtime requires a memory-safe execution environment. Instead of relying on manual `free()` operations, memory is managed by a Mark-and-Sweep Garbage Collector (GC). 

Since NewCP (the Rust-based runtime and JIT) aims to emulate the BlackBox Component Pascal semantics accurately, we will implement a GC adopting BlackBox's hybrid approach:
- **Conservative on the Stack:** The stack and CPU registers are scanned conservatively. Any value that *looks* like a heap pointer is treated as one. This frees the LLVM code generator from the immense complexity of emitting precise stack maps.
- **Precise on the Heap:** Objects allocated on the heap carry a type tag (`TypeDesc`) which precisely describes the byte offsets of all pointers within that object.
- **Precise on Globals:** Loaded modules provide metadata (`varBase` and an array of pointer offsets) mapping exactly where pointers live in their global data segment.

This document serves as the implementation guide for the `newcp-runtime` GC and its interaction with `newcp-llvm`.

## 1. Memory Organization

### 1.1 Clusters and Blocks
To support conservative stack scanning and efficient sweeps, memory is requested from the OS in large contiguous chunks called **Clusters** (e.g., using `VirtualAlloc` on Windows or `mmap` on POSIX). 
- A Cluster is subdivided into **Blocks** (objects or free space).
- The GC maintains a list of active Clusters. Given any random address `p`, the GC can quickly determine if `p` falls within an active Cluster. If it does not, it is ignored (crucial for conservative stack scanning).

### 1.2 Block Headers and `NEW`
When `SYSTEM.NEW` or `NEW()` is invoked, the compiler lowers this to a call like `Kernel.NewRec(tag)`.
The GC allocates a Block and immediately prefixes the returned user pointer with a header. 
In a 64-bit environment, the header is typically:
- `-8` offset: The `Tag` (Pointer to `TypeDesc`). The lowest bits of this tag pointer can be used to store flags (like the `Mark` bit) because `TypeDesc` addresses are aligned.

```rust
#[repr(C)]
pub struct BlockHeader {
    pub tag: *const TypeDesc, // Contains TypeDesc ptr + Mark bit in LSB
}
```

## 2. Type Descriptors (`TypeDesc`)

Every heap-allocated object (record, array) requires a `TypeDesc` so the GC knows how to trace its children. `newcp-llvm` must synthesize these and pass them to the allocation routine.

The `TypeDesc` must contain:
1. The size of the object.
2. An owning module pointer (or null for built-in types).
3. An optional finalizer function pointer; null when the type needs no cleanup.
4. An array of pointer offsets (`ptroffs`), indicating where pointers reside in the payload. The array is terminated by a sentinel (e.g., `-1`).

```rust
#[repr(C)]
pub struct TypeDesc {
    pub size: isize,
    pub module: *const ModuleDesc,
    pub finalizer: Option<unsafe extern "C" fn(*mut u8)>, // null = none
    pub ptroffs: [isize; 0], // Dynamically sized array of offsets
}
```

If present, the finalizer is invoked exactly once, on the dying block's payload pointer, **before** sweep zeroes the payload and links the block onto the free list. Finalizers must not allocate, must not retain the pointer, and must not perform GC-visible work.

When marking an object, the GC reads the object's `Tag`, finds the `TypeDesc`, and iterates `ptroffs`. For every offset `o`, it reads the pointer at `ObjectAddress + o` and recursively marks it.

## 3. Global Roots (The Module Graph)

The BlackBox runtime avoids scanning arbitrary BSS/Data segments. Instead, the module loader maintains a linked list of loaded modules (`modList`).

Each `ModuleDesc` exposes:
- `varBase`: The start address of the module's contiguous global data struct.
- `ptrs`: An array of local byte offsets identifying where heap pointers live within `varBase`.

During the Mark phase, the GC iterates the `modList`. For each module `M`:
```rust
for &offset in M.ptrs {
    let ptr = *(M.varBase + offset) as *const u8;
    if !ptr.is_null() {
        mark(ptr);
    }
}
```

*Codegen Requirement:* `newcp-llvm` must pack all of a module's global mutable variables into a single LLVM `%ModuleData` struct. It must calculate the layout offsets for any pointer fields and emit them as a global constant array, which is then linked to the `ModuleDesc`.

## 4. Stack Roots (Conservative Scanning)

The stack contains local variables, parameters, and temporary registers spilled by LLVM. 

### 4.1 `baseStack`
When the thread starting the Component Pascal runtime domain is launched, its stack pointer is recorded as `baseStack`. The stack grows downwards (on x86/x64).

### 4.2 Scanning Procedure
When `Collect()` is invoked:
1. The current stack pointer (`SP`) is captured. Register contents are spilled to the stack (e.g., using `setjmp` or inline assembly) to ensure they are scannable.
2. The GC iterates word-by-word from `SP` up to `baseStack`:
   ```rust
   let mut current = sp;
   while current < baseStack {
       let val = *(current as *const usize);
       if is_valid_heap_pointer(val) {
           mark(val as *mut u8);
       }
       current += std::mem::size_of::<usize>();
   }
   ```

### 4.3 Validation `is_valid_heap_pointer`
Because the stack contains integers and floats, many values will look like pointers. A value is only traced if:
1. It is properly aligned (e.g., 8-byte or 16-byte aligned).
2. It falls within the address range of a known allocated Cluster.
3. The address actually points to the payload of a valid Block header. (The GC can verify this by checking if the `Tag` block header is valid).

*Codegen Requirement:* `newcp-llvm` does not need to emit stack root maps. It can use standard `alloca` for local variables. However, any pointer passed to the runtime must point exactly to the start of the object to be recognized properly during stack scanning.

## 5. The Garbage Collection Cycle

The `Collect()` operation has two main phases:

### Phase 1: Mark
1. **Initialize:** Clear all mark bits.
2. **Mark Stack:** Conservatively scan from `SP` to `baseStack`. Mark found objects.
3. **Mark Globals:** Iterate `modList` and mark precise pointers via `varBase` + `ptrs`.
4. **Trace:** Process the mark queue. For every marked object, use its `TypeDesc.ptroffs` to find inner pointers and mark them. 
   *(Note: BlackBox originally used pointer-reversal to avoid allocating a mark stack, but in NewCP, an explicit vector in Rust or pointer-reversal can be used based on implementation constraints).*

### Phase 2: Sweep
1. Iterate over all Clusters.
2. Linearly walk through all Blocks in each Cluster.
3. If a block is Marked:
   - Clear the Mark bit.
4. If a block is Unmarked:
   - Call finalizers if a finalizer is registered for this `TypeDesc`.
   - Add the block's memory to the Free List to be reused by subsequent `NEW()` calls.

## 6. Implementation Milestones for `newcp-runtime`

1. **Allocator Core:** Implement Clusters, Blocks, and a basic Free List allocator (`alloc_block(size)`).
2. **Metadata Structures:** Define Rust structs mapping 1:1 with CP's `TypeDesc` and `ModuleDesc`.
3. **Conservative Scanner:** Write the OS/Arch-specific assembly to capture registers and scan the stack limits.
4. **Mark and Sweep:** Implement the trace loop utilizing `TypeDesc` and `Module` offsets, and the sweep loop reclaiming unmarked blocks.
5. **JIT Interface:** Expose `NewRec(tag: *const TypeDesc) -> *mut u8` as an exported symbol to the ORC JIT so compiled LLVM code can call it.

## 7. Multi-thread roadmap

The MVP runtime is single-threaded by deliberate choice (BlackBox's model). The design and implementation are forward-compatible with a future multi-threaded runtime; this section records the contract so that work added today does not have to be revisited.

### 7.1 What stays the same

- `BlockHeader`, `TypeDesc` (including the `finalizer` slot), `ModuleDesc` ABI.
- `Cluster` data structure: bitmap-indexed block-starts, free-list, sweep + coalescing, finalizer dispatch.
- `mark_object` work-stack tracer.
- `resolve_heap_ptr` interior-pointer resolution.
- The `__newcp_*` JIT-callable ABI as a whole.

### 7.2 Codegen guardrails (do today)

- **All managed allocation funnels through `__newcp_new_rec`.** No other crate touches `GcState` directly. Future TLABs and allocation sampling slot in by replacing this function body without touching call sites.
- **Emit a `__newcp_safepoint()` poll at every loop back-edge and at function entry.** Today the symbol is a no-op; the call cost is one indirect call and is eliminated by the linker / inliner. When stop-the-world support lands, the poll body is filled in without touching any generated code.
- **Finalizers are native-only.** They must not allocate, must not retain the payload pointer, and must not re-enter managed code. This rules out the JVM-style "finalizer thread" trap.
- **The `owner_thread` debug assertion in `gc.rs` is an MVP-internal guard**, not an architectural commitment. It will be replaced by per-thread `MutatorState` registration.

### 7.3 What changes for multi-thread (future work, in priority order)

1. **Per-thread state.** Move `base_stack` and SP capture into a `thread_local!` `MutatorState`. `GcState` keeps a list of registered mutators. Each managed thread registers on entry and unregisters on exit.
2. **Cooperative safepoints.** The `__newcp_safepoint` body becomes a load + branch on a global "GC requested" flag; on a hit, the mutator spills registers, marks itself parked, and waits on a condvar. Once every mutator is parked, the GC thread runs the cycle against guaranteed-quiescent stacks.
3. **TLABs (only if profiling shows `NEW` contention).** Each thread carves a small bump-pointer slab from a cluster under the global lock; subsequent allocations within that slab are lock-free.
4. **Finalizer policy.** Run finalizers on the GC thread (or whichever thread triggered the cycle). Avoid a dedicated finalizer thread.

The heap-representation work (items 2/3 of §5) is reusable as-is. The expensive new work is the safepoint protocol (item 7.3.2), which is unavoidable in any concurrent design.
