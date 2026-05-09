# OOP runtime status — pointer-aliased records, virtual dispatch

## What works today

- **Sema** auto-dereferences pointer aliases for field and method lookup.
  `TYPE Foo = POINTER TO FooDesc; (f: Foo) M*(): T;` now resolves
  `f.value` and `f.M()` cleanly through `lookup_record_member`.
- **IR** lowers `b.Method(args)` into proper vtable dispatch when `b` is
  pointer-aliased to a record with methods. The bound-proc detector runs
  `normalize_designator` first so the parser's greedy `module.name`
  packing of `b.Method` is unpacked into a base-plus-field designator;
  `method_slot_in_vtable` and `count_vtable_slots` resolve through the
  pointer alias to the underlying record so vtable slots line up.
- **`Instr::New`** allocates tagged records via `__newcp_new_rec(typedesc)`
  for record types that have methods. The runtime sets up the BlockHeader
  so `obj_ptr - 16` is the TypeDesc tag and the GC can trace the object.
  For records without methods the lowering falls back to `__newcp_sys_new`
  (no header) so non-OOP code (`Mod/Tests/Pointers.cp`) keeps working.
- **LLVM IR** for the dispatch path is textbook:

  ```llvm
  %hdr_ptr        = getelementptr i8, ptr %obj, i64 -16
  %tag            = load i64, ptr %hdr_ptr            ; BlockHeader.tag
  %desc_ptr       = inttoptr (and tag, ~1) to ptr     ; clear GC mark bit
  %vtable_field   = getelementptr i8, ptr %desc_ptr, i64 32   ; TypeDesc.vtable
  %vtable_ptr     = load ptr, ptr %vtable_field
  %fn_ptr         = load ptr, ptr (vtable_ptr + slot*8)
  call ret_ty %fn_ptr(%obj, args...)
  ```

  All of this is verified at the `dump-llvm` level (no JIT) for both
  `Mod/Tests/PtrMethod.cp` (concrete-only dispatch) and
  `Mod/Tests/AbstractDispatch.cp` (abstract-base virtual dispatch).

- **Allocation works at runtime.** `ptr_alloc_no_dispatch` and
  `ptr_alloc_block_header_tag_is_typedesc` both pass, confirming that
  `__newcp_new_rec` returns a payload whose `BlockHeader.tag` (at
  `obj - 16`) is the TypeDesc address.

## What's blocked

- **MCJIT does not relocate function-pointer constants in vtable
  initializers.** A `private`/`internal` constant of the form

  ```llvm
  @BoxDesc.vtable = internal constant [N x ptr] [ptr @BoxDesc_Set, ...]
  ```

  is allocated by MCJIT but the slot bytes stay zero — the relocation
  for `@BoxDesc_Set` is never applied. At runtime,
  `vtable[0] == 0`, the indirect call jumps to address 0, and the
  process segfaults.

  Confirmed empirically:
  - `BlockHeader.tag` (at `obj - 16`) is non-zero — TypeDesc address is
    correctly written by `__newcp_new_rec`.
  - `TypeDesc.vtable` (at `tag + 32`) points at a valid heap address —
    the `@BoxDesc.desc → @BoxDesc.vtable` reference IS relocated (data-
    pointer to data-pointer works).
  - `vtable[0]` reads as `0x0` — the function-pointer reference inside
    the vtable's array initializer is NOT relocated.

  So the relocator handles `ptr → data` references in const initializers
  but not `ptr → function`. This is a known MCJIT pain-point.

## What I tried

- **Linkage variants**: `Private` → `Internal` → `External`. None of them
  change the behavior. MCJIT allocates the global at a valid address;
  the array contents stay zero.
- **Mutable global + post-JIT patching**: declare the vtable as a regular
  `global` (not `constant`), then after engine creation walk each
  `*.vtable` global, query `LLVMGetGlobalValueAddress` via raw FFI, and
  write the function addresses into the storage at runtime. The address
  resolution works, but `engine.get_function_address("BoxDesc_Set")`
  returns "Function not found in ExecutionEngine" — MCJIT doesn't know
  about the symbol because nothing in the live IR references it
  directly.
- **`add_global_mapping` redirect**: bind the LLVM vtable global to a
  Rust-allocated buffer pre-engine-creation. MCJIT ignores the override
  for definitions (it only honors `add_global_mapping` for declarations
  with no body or initializer).

## Plausible fixes (in increasing scope)

1. **Force-emit method functions.** Mark each method function as
   `appending` in `@llvm.used`. This survives optimization AND ensures
   MCJIT keeps the symbol findable for `get_function_address`. Then
   the post-JIT patching path becomes viable. `inkwell` 0.9 doesn't
   expose `@llvm.used` directly, but it can be built with
   `Module::add_global` of a `[N x ptr]` constant of section
   `"llvm.metadata"`.
2. **Switch from MCJIT to ORC v2.** ORC's symbol table behavior is
   more predictable for cross-module function references in constants.
   `inkwell` has partial ORC support; would be a bigger refactor.
3. **Move dispatch to a runtime helper.** Instead of inlining the
   `obj → tag → vtable[i]` chain, emit `__newcp_dispatch(obj, slot,
   args...)` and have a Rust-side vtable registry. This gives up
   single-call dispatch in exchange for sidestepping MCJIT entirely.
   Would slow every method call by one extra call frame.

For the **Files port** (the original motivator): when the runtime
dispatch lands, the `Files.cp` interface module + `HostFiles` concrete
subclasses can be ported faithfully (Path A in
`files_module_investigation.md`). Until then, the `Files` port either
needs Path C (flat handle API, no OOP), or to wait.

## Tests in the corpus

Working today (in `tests/newcp-tests/src/lib.rs`):

| Test | What it asserts |
|---|---|
| `ptr_alloc_no_dispatch` | NEW + write field + read field round-trips through `__newcp_new_rec` |
| `ptr_alloc_block_header_tag_is_typedesc` | `BlockHeader.tag` at `obj - 16` is the TypeDesc address |

Fixtures kept as future regression tests (compile cleanly, JIT crash
expected until MCJIT fix lands):

- `Mod/Tests/PtrSet.cp` — `Run`: `b.Set(42)` followed by `b.value` read
- `Mod/Tests/PtrMethod.cp` — `Run`: `b.Set(42); b.Get()`
- `Mod/Tests/AbstractDispatch.cp` — `TestSquare`/`TestCircle`: virtual
  dispatch through abstract pointer base
