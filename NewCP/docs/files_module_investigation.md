# Files module — port status

## Current state — fully landed, end-to-end working

All four layers compile, JIT-load, and round-trip real bytes through
`std::fs`. The full
`HostFiles.theDir.This(path).New(loc, FALSE).NewWriter(NIL).WriteByte(0AAX)`
chain — Locator, File, Reader, Writer, Directory all dispatched
through their Files abstract bases — runs end-to-end and is verified
by the test suite.

| Layer | File | Status |
|---|---|---|
| Rust runtime | [src/newcp-runtime/src/host_file_sys.rs](../src/newcp-runtime/src/host_file_sys.rs) | ✅ 13 C-ABI shims over `std::fs` + handle table. |
| CP definition | [Mod/HostFileSys.cp](../Mod/HostFileSys.cp) | ✅ Flat-API definition module. |
| CP abstract | [Mod/Files.cp](../Mod/Files.cp) | ✅ Faithful port of BlackBox `System/Mod/Files.odc` — `Locator`, `File`, `Reader`, `Writer`, `Directory`, all 22 abstract methods, plus `InitType` and `SetDir`. |
| CP concrete | [Mod/HostFiles.cp](../Mod/HostFiles.cp) | ✅ Five concrete subclasses (`StdLocator`, `StdFile`, `StdReader`, `StdWriter`, `StdDir`) overriding the Files abstract surface. |
| CP test | [Mod/Tests/HostFilesRoundTrip.cp](../Mod/Tests/HostFilesRoundTrip.cp) | ✅ Loader-side sema clean; JIT-loads + executes. |
| Rust test | `tests::host_files_*` in [tests/newcp-tests/src/lib.rs](../tests/newcp-tests/src/lib.rs) | ✅ All 7 pass: `host_files_diag_this`, `host_files_diag_open`, `host_files_diag_open_direct`, `host_files_diag_flat_open`, `host_files_write_then_read_byte`, `host_files_write_then_read_bytes`, `host_files_length_after_write`. |

## What works

- **OS file I/O surface.** `HostFileSys.{Open, Close, ReadByte,
  WriteByte, ReadBytes, WriteBytes, Length, Pos, SetPos, Flush, Exists,
  Delete, Rename}` are exposed as JIT-resolvable symbols backed by
  `std::fs::File` and a Rust handle table. UTF-32 paths from CP
  decode at the FFI boundary. `cargo test -p newcp-runtime
  host_file_sys` confirms real round-trip.

- **Files.cp parses, sema-clean, JIT-loads.** Abstract record types
  (`LocatorDesc`, `FileDesc`, `ReaderDesc`, `WriterDesc`,
  `DirectoryDesc`) and 22 abstract methods all declare without error.
  The module body initialises `objType` / `symType` / `docType` to
  literal type strings.

- **HostFiles.cp compiles via `cargo run -p newcp-driver -- dump-llvm`.**
  All five concrete subclasses (`StdLocator`, `StdFile`, `StdReader`,
  `StdWriter`, `StdDir`) lower to LLVM IR with vtables.

## Sema blockers

The five blockers originally documented here have all been closed in
the `newcp-sema` cross-module resolution layer. The infrastructure
change: `Analyzer` now keeps an `imported_modules: HashMap<String,
Vec<SemanticSymbol>>` populated by recursively analysing each
imported `.cp` source. A post-processing step (`qualify_local_named_refs`)
rewrites every internal `Named { module: None, name: T }` reference
in an imported symbol to `Named { module: Some(<that module>), name:
T, kind: Imported }` so the same record type carries the same
canonical identifier on both sides of an inheritance edge.

Status of each:

1. **Method override across modules.** ✅ Fixed. New helper
   `has_inherited_method_anywhere` walks the inheritance chain
   through both local declarations and the imported-record method
   tables; the "must use NEW" diagnostic is suppressed when an
   ancestor in any module declares the same name. Verified by
   `xmod_subtype_assignment` (the test exercises an override of
   `XmodSubtypeBase.Greet` declared on the imported abstract base).

2. **Subtype assignability across modules.** ✅ Fixed. With imported
   record bases now carrying their original module qualification,
   `record_type_extends` walks across the inheritance chain
   correctly, so `RETURN <local_subclass_ptr>` typechecks against
   `expected: imported:<base_pkg>.<base_alias>`. Verified by
   `xmod_subtype_assignment`.

3. **Inherited abstract-base field access through pointer alias.**
   ✅ Sema half fixed by Blocker 5 (cross-module alias resolution
   makes the imported record's fields visible to
   `lookup_record_member`). An IR-layer follow-up remains —
   accessing the inherited field still fails at codegen with
   "unsupported cast from i64 to opaque:field:res" — but that's a
   lowering-side issue, not a sema gap.

4. **Type guard on pointer-aliased types.** ✅ Fixed.
   `validate_type_test_operands` now accepts targets of the form
   `POINTER TO Record` and reduces to the same extends-check on the
   underlying record type. Verified by all of the HostFiles
   `loc(StdLocator)` patterns now passing sema.

5. **Typedef compatibility across modules.** ✅ Fixed.
   `resolve_named_type_alias` now handles `Named { module: Some(m),
   kind: UserDefined | Imported }` by looking the type up in the
   imported module's symbol table. Verified by
   `xmod_type_alias_passes_array_of_char_through_imported_typedef`.

## Remaining items before HostFiles loads end-to-end

The three originally-documented items have all been resolved:

- **IR/codegen for inherited cross-module field access.** ✅ Fixed.
  `flatten_sem_type_fields` and `flatten_fields_for_ir_type` both
  now follow imported pointer-aliased types and pull the imported
  module's record fields. Verified by
  `xmod_inherited_field_access_through_pointer_alias` (assigns and
  reads `s.res` where `res` lives on the imported abstract base).

- **Integer literal → narrower integer assignment.** ✅ Fixed. New
  helper `integer_literal_fits_target` short-circuits the rank-based
  compatibility check at both the assignment-statement and
  argument-passing sites: a literal whose value fits the target
  integer type is accepted regardless of the literal's static
  type. Verified by `int_literal_narrows_to_byte` and
  `int_literal_narrows_to_shortint`.

- **`SHORT(...)` chain length.** ✅ HostFiles updated to chain
  three SHORTs (`SHORT(SHORT(SHORT(b)))`) for INTEGER → BYTE,
  matching NewCP's rank chain Integer → IntShort → ShortInt → Byte.

## Cross-module IR-layer fixes (now landed)

Driving the full HostFiles round-trip surfaced several IR-side gaps
beyond the original sema work. All three are now closed:

1. **Method-call argument lowering for fixed arrays / open arrays.**
   ✅ Fixed. Method-call arg lowering used to call `lower_expr`
   on each argument, which loads a fixed-size array as a value and
   doesn't emit the open-array fat-pointer `(ptr, len)` decomposition.
   `lower_call_args` was refactored to delegate to a shared
   `lower_args_with_signature(args, modes, types)` that consults
   the procedure's flattened parameter modes and types; method-call
   lowering now feeds it the imported method's signature so the same
   open-array, VAR-mode, fixed-array-by-reference, and SHORTCHAR
   widening rules apply uniformly.

2. **Cross-module vtable seeding for inherited concrete methods.**
   ✅ Mitigated by `__newcp_unimpl_method_trap`. When a HostFiles
   record extends a Files abstract base, the vtable's seed slots
   reference methods whose bodies live in Files' JIT module. The
   patch step in `jit::from_module` fills those slots with a
   Smalltalk-style `doesNotUnderstand:` stub that aborts with a
   descriptive message instead of jumping to address 0. Concrete
   subclasses that override the slot still get their own function
   pointer; only genuinely-unbound inherited slots ever reach the
   trap, and only when called.

   The proper long-term fix (cross-linking the slot to the defining
   module's compiled function address, or emitting a forwarding
   stub) remains a follow-up but is no longer load-bearing.

3. **Cross-module pointer-alias resolution in the IR layer.** ✅
   Fixed. `resolve_named_as_ptr_ir_type` now handles `"Module.Type"`
   by loading the imported module's sema and qualifying any internal
   `Named { module: None, name: T }` references with the importing
   module name (mirrors the sema layer's
   `qualify_local_named_refs`). This is what lets
   `sl: HostFiles.StdLocator` properly act as a pointer alias —
   field access `sl.path` now correctly emits a Load + GEP rather
   than treating the local var as an inline struct.

4. **Modal `Open` truncation.** ✅ Fixed. `host_file_sys_open` now
   truncates on `MODE_READ_WRITE` so `Directory.New` semantics
   (replace, not append) match BlackBox behaviour. Without this,
   bytes from a previous test run leaked into `Length()` results.

5. **Open-array fat-pointer ABI in the runtime FFI shims.** ✅
   Fixed. The `host_file_sys_*` functions now accept the hidden
   `_path_len` / `_buf_len` argument that CP's `IN ARRAY OF CHAR`
   and `VAR ARRAY OF BYTE` parameters pass before the trailing
   `mode` / `len` arg.

## Working today (test-suite verified)

157 / 157 passing. All `host_files_*` tests run end-to-end; cross-
module sema fixtures cover the underlying compiler features.

| Test | What it asserts |
|---|---|
| `xmod_type_alias_passes_array_of_char_through_imported_typedef` | `Files.Name` (= `ARRAY 16 OF CHAR`) accepted where `ARRAY OF CHAR` expected |
| `xmod_subtype_assignment` | `RETURN <local subclass>` typechecks against `imported:<base>`; cross-module override detected |
| `xmod_inherited_field_access_through_pointer_alias` | `s.res` reads/writes the inherited cross-module field |
| `int_literal_narrows_to_byte` | `x := 0` and `x := 200` for `x: BYTE` accepted |
| `int_literal_narrows_to_shortint` | Same for `SHORTINT` |
| `host_files_diag_this` | `HostFiles.theDir.This(path)` — virtual dispatch on imported receiver |
| `host_files_diag_open` | `loc(HostFiles.StdLocator)` type-guard + `sl.path` cross-module field read + flat `Open` |
| `host_files_diag_open_direct` | Local `path` to flat `Open` (open-array fat-pointer ABI) |
| `host_files_diag_flat_open` | `HostFileSys.Open(path, mode)` direct CP-side call |
| `host_files_write_then_read_byte` | Full Locator → File → Writer → Reader chain through Files abstract bases; round-trips a single byte through `std::fs` |
| `host_files_write_then_read_bytes` | Same with bulk `WriteBytes` / `ReadBytes` (open-array `BYTE`) — verifies all 8 bytes round-trip |
| `host_files_length_after_write` | `f.Length()` (abstract method returning `INTEGER`) through `Files.File` |

## Other minor follow-ups

- **`String literal := array_of_CHAR`** in `Files.cp`'s body —
  `objType := "ocf"` etc. compiles but the codegen emits a single
  `store ptr ...` instead of a 16-element memcpy, so consumers
  reading `Files.objType` will get pointer bits not characters.
  (Blocked the same way `Type$` is, see next item.)

- **`expr$` (string-length operator) on `Type` value.**
  `f.type := type$` errors as `"assignment type mismatch: expected
  type:Type, found CHAR"`. The `$` operator's result type is
  miscomputed for value-typed CHAR arrays. Worked around by writing
  `f.type := type` (skips the trailing-zero crop, which is fine for
  this use).

- **Inherited BOOLEAN field through abstract-base pointer.**
  `r.eof` on `Files.Reader` (`eof` declared on `Files.ReaderDesc`)
  triggers a codegen panic: the IR loads `eof` as `ptr` but expected
  `IntValue`. Same root as item 3 in the sema blockers, but
  manifests as a codegen-side mismatch when the call doesn't go
  through the type-checker.

- **Method call returning INTEGER through abstract-base pointer.**
  `f.Length()` (`Length` returns `INTEGER` on `Files.FileDesc`) hits
  `"unsupported cast from PointerType to i64"`. Same family of
  abstract-base-method-result lowering issue.

- **Empty `RECORD (Base) END`** seems to skip TypeDesc emission and
  fails at `Instr::New`. Worked around in `HostFiles.StdDirDesc` by
  adding a placeholder field.

## Recommendation

The Rust runtime and the CP definition module are good to land as-is
— `HostFileSys` is a useful flat-API primitive on its own (similar to
how `Console` is the layer below the eventual `Stores`-style text
output). The OOP wrappers (`Files.cp`, `HostFiles.cp`) should also
land, even though the loader rejects them today, so the work is
preserved and resumes naturally once the sema gaps above are closed.

The sema work needed before `Files` is callable from a real consumer
module is one focused effort on cross-module subtype + override
resolution in `newcp-sema`. It's the natural next target now that
the JIT vtable path works.

---

## Appendix: BlackBox surface for reference

(Original analysis — kept here for context.)

`Files.odc` is **almost entirely abstract** — type/method declarations
and zero file I/O logic. The actual implementation lives in
`HostFiles.odc` (~40 KB, 50 procedures, 37 distinct Win32 calls).

### Concrete subclasses in BlackBox

| Concrete | Extends | Role |
|---|---|---|
| `StdLocator` | `Files.Locator` | filesystem path holder |
| `StdFile` | `Files.File` | open file handle + buffer |
| `StdReader` | `Files.Reader` | sequential reader |
| `StdWriter` | `Files.Writer` | sequential writer |
| `StdDir` | `Files.Directory` | the singleton filesystem directory |

### Win32 calls in BlackBox HostFiles

```
CreateFileW   ReadFile   WriteFile   CloseHandle   FlushFileBuffers
SetFilePointer   GetFileSize   GetFileAttributesW   SetFileAttributesW
DeleteFileW   MoveFileW   CreateDirectoryW
FindFirstFileW   FindNextFileW   FindClose
GetFileTime   FileTimeToSystemTime   GetTempPathW   GetTickCount
GetVolumeInformationW   GetDriveTypeW   ExpandEnvironmentStringsW
GetCommandLineW   GetModuleFileNameW   GetLastError
```

All replaced in NewCP by `std::fs` calls in
[host_file_sys.rs](../src/newcp-runtime/src/host_file_sys.rs).

### Who depends on Files

`Files` has fan-in 58 ([yaml_module_tree.md](yaml_module_tree.md)).
Top consumers: `Stores`, `Documents`, `Sequencers`, `Converters`,
`Dialog`, `StdLoader`, `StdInterpreter`, `StdDialog`, every `Host*`
module, 14 of the 34 `Dev/*` IDE tools, `FormGen`, `OleStorage`.
