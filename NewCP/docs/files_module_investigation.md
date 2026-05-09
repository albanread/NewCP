# Files module — port investigation

## Summary

`Files` is a high-fan-in interface module (58 importers, second after Views/Dialog/Ports/Stores). Porting it unblocks `Stores`, `Documents`, `Converters`, `StdLoader`, all `Dev/*` IDE tools, all `Host/*` UI integration, and substantial parts of `Std/*` and `Form/*`.

The module is unusual: **the BlackBox `Files.odc` is almost entirely abstract** — 70 lines of type/method declarations and zero file I/O logic. The actual implementation lives in `HostFiles.odc` (~40 KB, 50 procedures, 37 distinct Win32 calls). NewCP can choose how to split this work.

## What `Files.odc` actually is

Source: [`YAML/System/Mod/Files.odc.yaml`](../../YAML/System/Mod/Files.odc.yaml).

```text
MODULE Files;
  IMPORT Kernel;

  CONST  shared, exclusive, dontAsk, ask, readOnly, hidden, system,
         archive, stationery   -- mode flags

  TYPE   Name       = ARRAY 256 OF CHAR
         Type       = ARRAY 16 OF CHAR
         FileInfo   = POINTER TO RECORD next, name, length, type, modified, attr END
         LocInfo    = POINTER TO RECORD next, name, attr END
         Locator    = POINTER TO ABSTRACT RECORD res END
         File       = POINTER TO ABSTRACT RECORD type, init END
         Reader     = POINTER TO ABSTRACT RECORD eof END
         Writer     = POINTER TO ABSTRACT RECORD END
         Directory  = POINTER TO ABSTRACT RECORD END

  VAR    dir, stdDir : Directory                  -- the active directory
         objType, symType, docType : Type         -- well-known file types

  -- 22 ABSTRACT methods on Locator/File/Reader/Writer/Directory:
  --   This, Length, NewReader, NewWriter, Flush, Register, Close, Closed, Shared,
  --   Base, Pos, SetPos, ReadByte, ReadBytes, WriteByte, WriteBytes,
  --   New, Old, Temp, Delete, Rename, SameFile, FileList, LocList, GetFileName

  PROCEDURE InitType(f, type)  -- 5 lines
  PROCEDURE SetDir(d)          -- 5 lines
BEGIN
  objType := Kernel.objType; symType := Kernel.symType; docType := Kernel.docType
END Files.
```

That's the entire module. No actual I/O.

## What `HostFiles.odc` is

Source: [`YAML/Host/Mod/Files.odc.yaml`](../../YAML/Host/Mod/Files.odc.yaml). 40 KB of CP. Five concrete record types extending the abstract ones:

| Concrete | Extends | Role |
|---|---|---|
| `StdLocator` | `Files.Locator` | filesystem path holder |
| `StdFile` | `Files.File` | open file handle + buffer |
| `StdReader` | `Files.Reader` | sequential reader |
| `StdWriter` | `Files.Writer` | sequential writer |
| `StdDir` | `Files.Directory` | the singleton filesystem directory |

`HostFiles` uses 37 Win32 calls:

```
CreateFileW   ReadFile   WriteFile   CloseHandle   FlushFileBuffers
SetFilePointer   GetFileSize   GetFileAttributesW   SetFileAttributesW
DeleteFileW   MoveFileW   CreateDirectoryW
FindFirstFileW   FindNextFileW   FindClose
GetFileTime   FileTimeToSystemTime   GetTempPathW   GetTickCount
GetVolumeInformationW   GetDriveTypeW   ExpandEnvironmentStringsW
GetCommandLineW   GetModuleFileNameW   GetLastError
... + helpers
```

Plus ~6 SYSTEM intrinsics (ADR, MOVE, etc.) and the `Files`/`Kernel` modules.

## Who depends on Files

`Files` has fan-in 58 ([yaml_module_tree.md](yaml_module_tree.md)). Top consumers:

- **System** : `Stores`, `Documents`, `Sequencers`, `Converters`, `Dialog`
- **Std**    : `StdLoader`, `StdInterpreter`, `StdDialog`, `StdApi`
- **Host**   : every Host*.odc except a handful
- **Dev**    : 14 of the 34 IDE tools (browser, debugger, compiler, packer, etc.)
- **Form**   : `FormGen`
- **Ole**    : `OleStorage`

## NewCP readiness

What's needed for a faithful port (in order of how blocking each is):

1. **`POINTER TO ABSTRACT RECORD`** — sema parses the keyword and rejects misuse, but no integration test verifies abstract types compile + JIT-execute. Listed under "Not yet verified" in [component-pascal-language-and-compiler-notes.md](component-pascal-language-and-compiler-notes.md). **Likely blocker.**

2. **Method dispatch through abstract base** — vtable + TypeDesc work for `EXTENSIBLE` records (verified by `dump_llvm_methods_emits_vtable_and_type_desc`). Whether a call through an `Files.Reader` variable resolves to the override on a `StdReader` instance hasn't been tested.

3. **Win32 FFI for file ops** — none exist yet. `WinApi` itself is a leaf module (no internal deps); a stub providing the 37 file/dir functions would be a few hundred lines of FFI declarations.

4. **`SYSTEM.MOVE` for byte-buffer copying** — already supported (`dump_llvm_system_move_emits_memmove`).

5. **Module body running at load** — already supported (just landed in [1ecfcf2](https://github.com/albanread/NewCP/commit/1ecfcf2)).

## Three port strategies

### A. Faithful — port both Files.cp and HostFiles.cp verbatim

Pros:
- BlackBox compatibility at the source level.
- Forces NewCP to gain real ABSTRACT support (good roadmap pressure).
- Sets the pattern for the other Host*.cp ports (HostFonts, HostPorts, …).

Cons:
- Largest scope: ~1000 lines of CP plus a substantial WinApi binding module.
- Blocks on ABSTRACT-record runtime support, which is unverified.
- Only works on Windows. Cross-platform requires reimplementing HostFiles for each OS.

Effort: weeks. Validates a lot of compiler features.

### B. Bridged — Files.cp interface, Rust HostFiles backend

Pros:
- Files.cp stays BlackBox-compatible (consumers see the right surface).
- HostFiles is portable (Rust `std::fs` instead of Win32).

Cons:
- Need a way for the Rust runtime to **register** a concrete CP-typed `Directory` instance whose abstract methods dispatch to Rust function pointers. NewCP doesn't have this mechanism.
- Either:
  (i) Build it: extend the runtime so Rust can synthesize a tagged record + vtable that the JIT recognises. Not trivial.
  (ii) Have a thin CP shim module (`HostFiles.cp`) that defines concrete subclasses whose method bodies are one-line `extern` calls to `__hostfiles_*` Rust functions. This works but each abstract method requires a CP wrapper line.

Effort: medium. (i) is research; (ii) is bookkeeping.

### C. Flat C-style — abandon BlackBox-compat, ship file I/O now

Pros:
- Smallest possible: `Files.cp` becomes a flat API (Open/Read/Write/Close with opaque integer handles), `HostFiles.rs` is the Console/Math template again.
- Works today with no missing compiler features.
- Cross-platform out of the box (Rust `std::fs` everywhere).

Cons:
- Breaks BlackBox compatibility. Modules like `Stores`, `StdLoader`, `Documents` would all need to be rewritten to use the flat API instead of method calls on `Files.Reader` / `Files.Writer`.
- Stuck with this divergence forever, or until the OOP path catches up.

Effort: small (1–2 days). Approach matches our `Math` / `SMath` / `Console` pattern.

## Recommendation

**Start with C (flat) under the name `Files`, then later add B-style abstract wrappers when ABSTRACT-record runtime support is verified.**

Reasoning:

1. The 58 downstream modules will all need to be ported anyway. They can be ported against either API surface. Picking the flat surface today doesn't lock anything out — when the OOP version exists, downstream callers can be migrated one at a time, or the flat API can be reimplemented as one-liner forwards to the OOP API.

2. The flat API is what NewCP can ship **this week**. The faithful port is a multi-week effort and requires committing to ABSTRACT runtime support first.

3. ABSTRACT support is going to need its own dedicated work (test fixtures, vtable through abstract base, type-test against abstract types, etc.). Doing that work in service of `Files` couples two big efforts; doing them separately keeps each one simple.

4. The flat surface validates the Rust-resident-module pattern under a substantially bigger workload than `Console` (file handles, error codes, persistent state, directory iteration). It's a useful stress test.

### Concrete first slice (Option C scope)

Define `Files.cp` as a `DEFINITION MODULE` declaring:

```text
TYPE Handle* = INTEGER;        (* 0 = invalid *)

(* file lifecycle *)
PROCEDURE Open*(IN path: ARRAY OF SHORTCHAR; mode: INTEGER): Handle;
PROCEDURE Create*(IN path: ARRAY OF SHORTCHAR): Handle;
PROCEDURE Close*(h: Handle);

(* read / write *)
PROCEDURE ReadBytes*(h: Handle; VAR buf: ARRAY OF BYTE; len: INTEGER): INTEGER;
PROCEDURE WriteBytes*(h: Handle; IN buf: ARRAY OF BYTE; len: INTEGER): INTEGER;

(* positioning *)
PROCEDURE Pos*(h: Handle): INTEGER;
PROCEDURE SetPos*(h: Handle; pos: INTEGER);
PROCEDURE Length*(h: Handle): INTEGER;

(* directory ops *)
PROCEDURE Delete*(IN path: ARRAY OF SHORTCHAR): BOOLEAN;
PROCEDURE Rename*(IN old, new: ARRAY OF SHORTCHAR): BOOLEAN;
PROCEDURE Exists*(IN path: ARRAY OF SHORTCHAR): BOOLEAN;

CONST modeRead* = 0; modeWrite* = 1; modeReadWrite* = 2;
```

Backed by `newcp-runtime/src/files.rs` exposing one `extern "C"` per declaration, using `std::fs::File` keyed off a side-table. ~200 lines of Rust.

### Tests to write

- `roundtrip` — Create + WriteBytes + Close + Open + ReadBytes + Close, assert content
- `positioning` — Open, SetPos, ReadBytes from middle, assert content
- `not_found` — Open(nonexistent), assert returns 0
- `delete` — Create + Close + Delete + Exists, assert false
- `length` — Create + WriteBytes + Length, assert N

Each one is the same shape as our `Math` smoke tests.

### Path to OOP later

When `ABSTRACT` runtime support lands:

1. Add `FilesOop.cp` (or rename current Files → FilesFlat and put the OOP version at Files) with the BlackBox interface.
2. Concrete subclasses (`StdReader` etc.) hold a `Handle` and forward to `FilesFlat.*`.
3. Existing flat-API consumers can either keep using FilesFlat or migrate to the OOP API one at a time.

This way no work is wasted: the flat API stays a useful primitive even after the OOP layer lands.

## Appendix: notes for either path

- **Path encoding.** BlackBox `Files.Name` is `ARRAY 256 OF CHAR` (UTF-32 in NewCP, was UCS-2 in BlackBox). Win32 wants UTF-16; std::fs wants `&Path`. The Rust side will need a UTF-32 → OS-native conversion at the FFI boundary.
- **`BYTE` type.** NewCP `BYTE` lowers to `U8` — directly compatible with Rust `u8`. `ARRAY OF BYTE` parameters use the same fat-pointer ABI as `ARRAY OF SHORTCHAR`, so no new ABI work.
- **Locator / Directory model.** BlackBox uses `Locator` to abstract over filesystem roots (current directory, app-relative, etc.). For a first slice we can just take absolute paths and skip Locator entirely; add it back when we wrap the flat API in OOP.
- **`Kernel.objType / symType / docType`.** Three short string constants consumed by the loader. Cheap to add to the Kernel runtime once we need them.
