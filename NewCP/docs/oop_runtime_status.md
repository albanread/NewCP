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
- **JIT dispatch now works at runtime.** Vtables are emitted as mutable
  globals with zero initializers, and after MCJIT materializes the module
  the JIT layer:
  - resolves each vtable global address via `LLVMGetGlobalValueAddress`
  - resolves each method body via `LLVMGetPointerToGlobal` on the concrete
    `FunctionValue`, avoiding MCJIT's unreliable name-based lookup for
    non-exported or generation-qualified methods
  - writes those addresses into the vtable slots in place

  This also handles exported methods correctly because the recorded slot
  metadata is normalized to the final emitted LLVM symbol name during
  procedure declaration.

  Verified end-to-end:
  - `ptr_set_probe_vtable_fn`
  - `ptr_method_box_set_get`
  - `abstract_dispatch_square`
  - `abstract_dispatch_circle`

## Root cause

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
  but not `ptr → function`. This is the underlying reason the original
  constant-vtable approach failed.

## Failed attempts

- **Linkage variants**: `Private` → `Internal` → `External`. None of them
  change the behavior. MCJIT allocates the global at a valid address;
  the array contents stay zero.
- **Mutable global + post-JIT patching via `get_function_address`**:
  writing slots post-JIT is the right shape, but MCJIT still refused to
  expose some method bodies through its symbol table. That variant failed
  with errors such as `Function not found in ExecutionEngine`.
- **`add_global_mapping` redirect**: bind the LLVM vtable global to a
  Rust-allocated buffer pre-engine-creation. MCJIT ignores the override
  for definitions (it only honors `add_global_mapping` for declarations
  with no body or initializer).

## Landed fix

The working combination is:

1. Emit each vtable as a mutable zero-initialized global.
2. Anchor method bodies with `@llvm.used` so optimization does not delete
  them.
3. Record vtable slot names using the final emitted LLVM symbol names.
4. After JIT creation and `add_global_mapping`, finalize the engine
  explicitly via `engine.run_static_constructors()` (we don't emit
  `llvm.global_ctors`, so this only triggers `MCJIT::finalizeObject()`
  — no user constructors run). This makes the materialization step a
  named operation rather than a side effect of the first address lookup.
5. Resolve each method via `LLVMGetPointerToGlobal` on the actual
  `FunctionValue` rather than via MCJIT's symbol table.
6. Patch the vtable storage in place.

This preserves the direct dispatch path in generated code:
`obj -> tag -> desc.vtable -> vtable[i] -> fn_ptr -> call`.

### Single-module-per-engine assumption

The current architecture creates one `ExecutionEngine` per
`OwnedJitModule` (no `engine.add_module(...)` path). Step 4 finalizes
exactly that one module, which is sufficient for everything we ship
today.

If incremental compilation is added later (multiple modules
JIT'd into the same engine in stages), MCJIT will **not** retroactively
re-finalize earlier modules when a new one is added; each newly-added
module must trigger its own finalization before its globals or
functions are queried by address. The natural place for this is in
whatever wraps `engine.add_module(...)` — call
`engine.run_static_constructors()` (or an equivalent
`finalizeObject`-bearing API) before doing any address-resolution or
vtable-patching for the new module.

## Remaining options

- **Switch from MCJIT to ORC v2** for a more principled JIT backend.
- **Move dispatch to a runtime helper** if future MCJIT edge cases still
  make direct patching too brittle.

For the **Files port** (the original motivator): the OOP runtime blocker is
gone for this dispatch path, so the faithful `Files.cp` interface module +
`HostFiles` concrete subclass port is now technically viable again.

## Tests in the corpus

Working today (in `tests/newcp-tests/src/lib.rs`):

| Test | What it asserts |
|---|---|
| `ptr_alloc_no_dispatch` | NEW + write field + read field round-trips through `__newcp_new_rec` |
| `ptr_alloc_block_header_tag_is_typedesc` | `BlockHeader.tag` at `obj - 16` is the TypeDesc address |
| `ptr_set_probe_vtable_fn` | `vtable[0]` is patched to a non-zero method address |
| `ptr_method_box_set_get` | concrete pointer-aliased virtual dispatch works end-to-end |
| `abstract_dispatch_square` | abstract-base dispatch resolves Square.Area |
| `abstract_dispatch_circle` | abstract-base dispatch resolves Circle.Area |
