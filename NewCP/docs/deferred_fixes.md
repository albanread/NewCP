# Deferred fixes

Items the codebase **knows about** but ships with a workaround instead
of the principled fix. Each entry records what the workaround is, why
it's defensible, and what closing the item would entail.

This file is the index — please add new items as you discover them
during ports. Don't put unrelated TODOs here; this is for places where
*shipping code* takes a deliberate shortcut.

---

## Compiler / language

### 1. Value-mode record/array params: rejected, not honoured

**Where**: `newcp-sema` in `collect_parameter_section_symbols`.

**Workaround**: Sema rejects value-mode parameters of record or
fixed-array type and prompts the user to pick `IN`/`VAR`/`OUT`
explicitly. Receivers (`(b: BoxDesc) Method ()`) are exempt.

**Why deferred**: NewCP passes records and arrays by reference at the
call ABI. The CP spec says value-mode parameters give the callee a
*private copy* that the caller can't see writes to. Honouring the
spec means emitting a defensive `memcpy` at function entry for every
value-mode record/array parameter. That's a real cost and no current
port needs it — every BlackBox idiom that wanted "private copy of a
record" used `IN` (read-only) anyway.

**Closing it**: in IR lowering, for any value-mode parameter whose
type is a record or fixed array, allocate a stack slot at function
entry and `memcpy` the parameter into it before any user code runs;
rebind the parameter symbol to point at the slot. Then sema can drop
the rejection check. Estimated cost: ~50 lines in `lower.rs` plus
a runtime memcpy intrinsic call (already have `__newcp_memcpy`).

### 2. `IN` writes through pointer-aliased dereferences are over-rejected

**Where**: `newcp-sema` in `validate_assignment_target`.

**Workaround**: Sema rejects *any* assignment whose root is an `IN`
parameter, including writes through a dereferenced pointer
(`p^.field := x` or `p.field := x` where `p` is a pointer alias).

**Why deferred**: CP's actual semantics for `IN` of a pointer type
are that the *pointer value* is read-only (you can't reassign `p`),
but writes through `p^.field` are allowed because the pointee is a
separate object. The conservative rule we ship today is what
BlackBox's compiler actually enforces for the modules in the corpus;
no real fixture needs the looser interpretation yet.

**Closing it**: in `validate_assignment_target`, walk the LHS
designator's selectors before deciding. If the chain encounters a
`Selector::Dereference` (or implicit auto-deref through a pointer
alias) before any field/index selector that would write, allow the
write. Costs careful attention to avoid loosening too much.

### 3. Cross-module inherited concrete methods reach a trap stub, not the real body

**Where**: `newcp-llvm` in `JitModule::from_module`.

**Workaround**: When a subclass extends an imported abstract base and
*does not* override an inherited concrete method, the vtable slot for
that method is filled with `__newcp_unimpl_method_trap` (Smalltalk-
style `doesNotUnderstand:`). Calling the slot aborts with a
descriptive message; not calling it stays inert. For HostFiles every
method is overridden, so the trap slots never fire.

**Why deferred**: The concrete method bodies live in the *defining*
module's JIT engine, not the subclass's. Routing the call there from
a different module's vtable requires either (a) a per-module
forwarding stub the JIT can write a function pointer into, or (b)
cross-linking the slot at finalization time using
`get_function_address` against the defining module's engine. (b) is
cleaner but needs a multi-engine address resolver.

**Closing it**: option (b). Add `JitModule::resolve_external_method_
addr(module: &str, llvm_name: &str) -> Option<usize>` that the
patch loop in `from_module` calls when `method_functions.get(name)`
returns None. Requires the loader to keep a `HashMap<String,
&JitModule>` of materialised modules so the resolver can find the
defining engine.

### 4. Multi-module-per-engine incremental compilation needs per-module finalisation

**Where**: `newcp-llvm` in `JitModule::from_module`.

**Workaround**: We ship single-module-per-engine. The explicit
`engine.run_static_constructors()` finalises that single module
before any address resolution happens.

**Why deferred**: Adding modules incrementally to a long-lived engine
is a feature we don't need today. Every `OwnedJitModule` owns its
own engine.

**Closing it**: when `engine.add_module(...)` is called for a second
module, call `engine.run_static_constructors()` again before that
module's address resolution. Document the contract on whatever wraps
`add_module`. See the comment block in `JitModule::from_module` for
the full caveat.

### 5. `expr$` (string-length operator) result type wrong for value-typed `ARRAY OF CHAR`

**Where**: `newcp-sema` (operator typing).

**Workaround**: In `Mod/Files.cp`'s `InitType`, the BlackBox source
has `f.type := type$;` (`$` strips trailing zeros). Sema reports
`assignment type mismatch: expected type:Type, found CHAR` for that
form. Worked around by writing `f.type := type` — drops the
truncation but the data round-trip is fine for the test fixtures.

**Why deferred**: No fixture *needs* the truncation behaviour today.
A string-array assignment without `$` copies the whole buffer
including padding, which is functionally indistinguishable for
length-checking consumers.

**Closing it**: in sema's expression-type inference for the unary
`$`, look at the operand's declared type. If it's `ARRAY n OF CHAR`,
the result type should be the same array type (not `CHAR`). This is
purely a type-checker bug; the IR layer would need no changes.

### 6. `String literal := array_of_CHAR` in module body emits `store ptr` instead of `memcpy`

**Where**: `newcp-llvm` codegen for module-body assignments.

**Workaround**: `Mod/Files.cp`'s body has
`objType := "ocf"; symType := "osf"; docType := "odc"`. The codegen
emits a single `store ptr ...` for each — storing the address of
the string global into the first 8 bytes of a 16-element CHAR array.
Consumers reading `Files.objType` get pointer bits, not characters.

**Why deferred**: Nothing in the corpus reads those globals yet.
When `Stores` lands and starts using `Files.objType` to dispatch
file types, this becomes load-bearing.

**Closing it**: detect `Statement::Assignment { target, value }`
where `target` is a fixed-CHAR-array designator and `value` is a
string literal of length ≤ array capacity. Emit `Instr::MemCopy`
from the literal global into the target instead of a store.
Already done at the procedure-body level (commit `8b6c306`); the
module-body path needs the same treatment.

### 7. Inherited BOOLEAN field through abstract-base pointer triggers IR/codegen mismatch

**Where**: `newcp-ir` field-access lowering or `newcp-llvm` emit.

**Workaround**: `r.eof` for `r: Files.Reader` (`eof` declared on the
abstract `Files.ReaderDesc`) used to crash codegen with "Found
PointerValue but expected the IntValue variant". The cross-module
field-access fix in the Files port (`flatten_sem_type_fields`
following imported pointer-aliased bases) closed the path that
triggered it. The `r.eof` test in `Mod/Tests/HostFilesRoundTrip.cp`
now passes.

**Why deferred**: marked here just to flag that the diagnostic was
historically misleading — the underlying issue was cross-module
field flattening, not BOOLEAN handling. Listed for future readers
who hit a similar "expected IntValue, found PointerValue" panic
elsewhere; the first thing to check is whether the flattening
follows pointer-aliased imported bases.

**Closing it**: nothing to do — already closed. Listed as a
historical-record entry.

---

## Runtime / host

### 8. `HostDateSys.GetUTCBias` returns 0

**Where**: `src/newcp-runtime/src/host_date_sys.rs`.

**Workaround**: The bias query always returns 0 (UTC = local time).
`HostDateSys.GetLocalTime` is therefore identical to `GetUTCTime` on
every platform. The Rust runtime test pins to UTC and the CP test
suite only checks year ranges, so nothing observes the wrong value.

**Why deferred**: `std::time` doesn't expose the local timezone
offset cross-platform. Fixing it portably means adding `chrono` as a
dependency or writing per-OS shims (`GetTimeZoneInformation` on
Windows, `localtime_r` via `libc` on Unix). Neither is justified by
a current port.

**Closing it**: one of:
- Add `chrono = "0.4"` and call `chrono::Local::now().offset()`.
- Per-OS implementation behind `#[cfg]`.

### 9. `HostDateSys.DateToString` formats are American-defaulted

**Where**: `src/newcp-runtime/src/host_date_sys.rs`.

**Workaround**: `DateToString` produces `"5/9/2026"`, `"May 9, 2026"`,
etc. without consulting the system locale. The tests pin to these
exact formats.

**Why deferred**: same as #8 — locale handling is platform-specific
and no port needs it yet.

**Closing it**: thread the OS locale through. Probably becomes
trivial once `chrono` is in (use `chrono::format::DelayedFormat`).

### 10. `HostFiles` simplifications vs BlackBox

**Where**: `Mod/HostFiles.cp`.

**Workarounds** (all minor):
- `Locator` carries a path string only; no "current directory" /
  "stationery folder" semantics. Pass absolute paths.
- `File.Register` is a no-op. BlackBox uses it to commit a temp file
  to its real name; `Directory.New` opens the target path directly.
- `FileList` / `LocList` / `SameFile` / `GetFileName` return NIL /
  FALSE stubs.
- `MODE_READ_WRITE` truncates on open (replace, not append) —
  matches `Directory.New` semantics but means callers can't append
  to an existing file via `New`.

**Why deferred**: every `host_files_*` round-trip test passes with
this surface. Real workloads (`Stores`, `Documents`) will add
demands one at a time.

**Closing it**: per-feature, as downstream consumers need each piece.

---

## Tooling

### 11. YAML text extraction loses identifier names

**Where**: `tools/extract_imports.py` / ad-hoc shell extraction during
ports.

**Workaround**: BlackBox `.odc.yaml` text fragments split between the
plain-attribute body and `attr: 3` highlight runs (the identifier
names). Naïvely concatenating only the `text` fields produces a
source file with `* = RECORD ... END;` instead of `Date* = RECORD ...
END;` — every type/proc/const name is missing. During the Files and
Dates ports, the workaround was to re-extract names by grepping for
`attr: 3` runs and splicing them back in by hand.

**Why deferred**: the extraction tools work for the import-graph
analysis they were written for. A proper text reconstructor is its
own small project.

**Closing it**: write `tools/extract_source.py` that walks the YAML
tree, concatenates text fragments in document order, and respects
`attr` highlight runs as additional text. ~100 lines of Python.

---

## Conventions for adding entries

Each entry follows the same template:

- **Where**: file/function the workaround lives in
- **Workaround**: what the code does today
- **Why deferred**: the cost-vs-need argument for not doing it now
- **Closing it**: concrete description of what the fix would be

Keep entries short and self-contained. If something deserves a
multi-page design discussion, give it its own doc and link it from
here.
