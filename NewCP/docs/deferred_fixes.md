# Deferred fixes

Items the codebase **knows about** but ships with a workaround instead
of the principled fix. Each entry records what the workaround is, why
it's defensible, and what closing the item would entail.

This file is the index — please add new items as you discover them
during ports. Don't put unrelated TODOs here; this is for places where
*shipping code* takes a deliberate shortcut.

---

## 🔴 Urgent

> Items here should jump the queue ahead of feature work — they're load-bearing
> CP idioms whose breakage is silent / unsafe rather than loud / contained.

*(Currently empty — #19 short-circuit was filed here, fixed, and the
entry retired.  The matrix probe `M_Expr_ShortCircuit_NilGuard` is
the regression target.)*

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

### 12. `SHORT()` IR lowering doesn't reflect NewCP's collapsed integer widths

**Where**: `newcp-ir` `lower.rs::lower_builtin_expr` "SHORT" arm.

**Workaround**: `SHORT(I64)` always emits a real `i64 → i32` truncation
in IR. NewCP maps both `LONGINT` and `INTEGER` to `i64`, so a
BlackBox-style `SHORT(longint_expr)` (semantically `LongInt → Integer`,
both `i64` in NewCP) erroneously truncates to `i32`. This shows up as
LLVM verifier failures at call sites whose parameter type is `INTEGER`
(`i64`) but whose argument has been silently narrowed.

**Why deferred**: SHORT's intent is sema-level, but the IR layer has no
sema-type-of-expression accessor today. A proper fix needs to plumb the
sema type of the argument expression into the lowerer's SHORT arm so it
can produce a no-op when narrowing between two semantic levels that
share an IR width (LongInt↔Integer).

**Closing it**: add a small `sema_type_of_expr` helper on `LowerCtx`
that infers the semantic type of an `Expr` against `module_symbols` /
local symbols (a thin re-implementation of sema's `infer_expr_type`,
or a public re-export from sema). At the SHORT arm, consult that
helper to choose the right IR cast — or no cast at all when both
sides land on the same IR width.

### 13. `Mod/Integers.cp` blocked on SHORT-chain mismatch (port stalled)

**Where**: `Mod/Integers.cp` (full BlackBox bignum module, lifted but
not yet compiling end-to-end).

**Workaround**: source committed; sema accepts it (after fixes in this
landing for type-alias resolution into builtins, OUT-pointer-to-array
indexing, value-mode `Buffer` params switched to `IN` with explicit
local copies in `KStep`/`AddToBuf`, MAX/MIN with user type alias args).
LLVM emit fails on calls like `New(SHORT(... + ENTIER(...)))` because
of item 12 above (the `SHORT` truncates `i64`→`i32` at IR level even
though `New`'s parameter is also `i64`).

**Why deferred**: the right fix is item 12 — making SHORT semantically
aware. Hacking the source to drop the SHORTs would diverge it from
the BlackBox original and lose the type-narrowing intent that other
back-ends rely on.

**Closing it**: implement item 12, then re-run `check-mod` /
`dump-llvm` and add `Integers` tests covering `Sum`, `Difference`,
`Product`, `Quotient`, `Power`, `ConvertFromString`/`ConvertToString`.

### 14. `NEW(record_field_pointer)` — IR can't resolve destination record type

**Where**: `newcp-ir` lowering of `Instr::New`, exercised by the
matrix probe `M_Method_On_RecordField` (currently
`#[ignore]`-flagged in `src/newcp-test-matrix/src/manifest.rs`).

**Workaround**: probe is shipped ignored with the reason string
pointing at this entry. Real code paths avoid the pattern by
NEW-ing into a local pointer first then assigning into the field.

**Why deferred**: surfaced by the matrix on its first run after the
strategy in `docs/test_matrix.md` landed; the framework ports
haven't tripped over it yet so it stays in the deferred list rather
than blocking active work.

**Closing it**: in IR `lower_new`, when the destination designator
ends in a field selector, follow the field's declared type to
resolve the record name instead of falling back to
`Opaque("new-ptr")`. Un-ignore the matrix probe to lock the fix in.

### 16. `IS` test against an uninstantiated record type crashes

**Where**: IR / runtime type-test fast path. Surfaced by the matrix
probes `M_AnyPtr_IS_Test` and `M_Expr_Pointer_IS_Test` (both
`#[ignore]`-flagged with this entry as their reason).

**Workaround**: probes shipped ignored. Real code paths so far have
only `IS`-tested against types that were also instantiated
elsewhere in the same translation unit, so their `TypeDesc` was
registered before the test ran.

**Why deferred**: surfaced by the matrix's first expansion pass.
Easy to write a reduction (a 20-line probe) but the fix sits in
the runtime type-test path — likely a NIL `TypeDesc` deref that
needs to be hardened, or codegen needs to ensure every declared
record type's `TypeDesc` is registered even when never NEW'd.

**Closing it**: harden `__newcp_type_test` (or the IR
`Instr::TypeCheck` lowering) so a NIL `TypeDesc` on the
right-hand-side comparison returns `false` instead of segfaulting.
Verify by un-ignoring both probes; their packed values are 110 and
1010 respectively.

### 17. `IN p: PointerAlias` dereference crashes at runtime

**Where**: parameter-access codegen. Surfaced by matrix probe
`M_Param_IN_Pointer_Deref` (`#[ignore]`-flagged with this entry).

**Workaround**: probe shipped ignored. Most real code uses `VAR`
or value-mode (or method receivers) when it wants to deref a
pointer in the callee.

**Why deferred**: the codegen for `IN <pointer-alias>` parameters
appears to misread the slot — treating it like a record value and
skipping the heap-pointer Load — and the resulting bad address
causes `STATUS_ACCESS_VIOLATION` on first field access. Shape is
similar to the method-dispatch-receiver fix landed earlier in
this round, but on the parameter-access path rather than the
receiver path.

**Closing it**: in `lower_designator` / param-slot lowering, when
the formal's IR type is `Ref(Named(N))` and `N` resolves to a
pointer alias (not a record), emit the load that fetches the heap
pointer before GEPing for the field. Then un-ignore the probe.

### 18. Indirect call through a procedure-typed parameter mis-types its args

**Where**: sema's type resolution for call expressions whose callee
is a procedure-typed parameter (not a local var or named
procedure). Surfaced by matrix probe `M_ProcType_Param_Callback`
(`#[ignore]`-flagged with this entry).

**Workaround**: probe shipped ignored. The other procedure-type
probe (`M_ProcType_IndirectCall`) assigns the proc-value to a
*local* before calling, which works — so production code paths
have a clean workaround: copy the param to a local and call
through that.

**Why deferred**: surfaced by the matrix on first expansion. Sema
reports `found unresolved:seed` for an argument that's a peer
parameter in the same procedure — so the lookup scope is broken
specifically when the call's callee is itself a procedure-typed
parameter (not when it's a local var of the same type).

**Closing it**: walk `lower_bound_proc_call_expr` / its sibling
indirect-call resolution and make sure the surrounding scope's
local symbols stay visible while the callee's signature is being
matched. Un-ignore the probe to confirm; its `Run` returns 121.

### 15. CHAR / SHORTCHAR width mismatch at call boundary (8 failures at `-O0`)

**Where**: `Mod/Strings.cp` and transitively `Mod/Math.cp`. Surfaced
by running the full test suite with `NEWCP_OPT=none`; the LLVM
verifier reports `Call parameter type does not match function
signature! i32 143 / i8` against `Strings$g1$RealToShortStrForm`'s
formal. Eight tests fail at `-O0`:
`math_exponent_decomposition`, `math_int_power_via_native_module`,
`math_pi_via_native_module`, `math_sqrt_via_native_module`,
`strings_real_to_short_str_round_trip`,
`strings_real_to_string_round_trip`,
`strings_short_str_to_real`, `strings_string_to_real_roundtrip`.

**Workaround**: default `-O2` lane silently truncates the `i32` to
`i8` so the bug never reaches the verifier. The suite passes at
`-O2` (247 tests green) but the latent ABI mismatch is real.

**Why deferred**: the test-matrix infrastructure that uncovered it
was the priority for the current landing. The fix is a localised
call-site widening / narrowing decision and belongs in its own
session so it can be measured against the `-O0` lane cleanly.

**Closing it**: walk the `RealToShortStrForm` call site (and any
others the `-O0` verifier flags) — the argument is almost
certainly a SHORTCHAR (`i8`) formal being supplied with a CHAR
(`i32`) value or vice versa. Either widen at the call site
(`Cast` IR), narrow in the prologue, or align the formal's declared
type. Re-run the suite at `NEWCP_OPT=none` until all 8 cases pass;
add the `NEWCP_OPT=none` lane to CI so the bug class stays out.

### 19. ~~Logical `&` / `OR` do not short-circuit~~ — FIXED

**Status**: closed. `newcp-ir/src/lower.rs::lower_short_circuit_boolean`
emits a CFG-shaped lowering — branch on the left operand, only
evaluate the right operand in the appropriate arm, phi-equivalent
via a synthetic `$lshort_<id>` stack slot at the merge.

**Regression coverage**: matrix probes
`M_Expr_LogicalAnd_ShortCircuit`, `M_Expr_LogicalOr_ShortCircuit`,
and `M_Expr_ShortCircuit_NilGuard` (the last one exercises the
`IF (p # NIL) & (p.field > 0) THEN` BlackBox idiom that was
silently dereferencing NIL before the fix).

### 20. Sema rejects subset/superset (`<=` / `>=`) on SET operands

**Where**: sema's relational-operator type table. Surfaced by
matrix probe `M_Expr_SET_Equality` (`#[ignore]`-flagged).

**Workaround**: probe ignored. Real code that wants a subset test
can use the equivalent `(a * b) = a` (intersection equals self).

**Why deferred**: CP §8.2.5 defines `s1 <= s2` as "s1 is a subset
of s2" and `s1 >= s2` as superset. The sema reports
`invalid operands for <=: SET and SET`. Just needs adding the
SET-SET pair to the relational-operator type table; the runtime
already supports the operation through `(a * b) = a` so no
codegen work is required.

**Closing it**: extend the relational-operator type rules in
`newcp-sema` to allow SET on both sides for `<=` / `>=` /
`<` / `>`, lowering to the subset/superset semantics. Un-ignore
the probe.

### 21. Multi-dimensional fixed-array indexing crashes codegen

**Where**: LLVM emit for `arr[i, j]` where `arr` is declared as
`ARRAY M, N OF T`. Surfaced by matrix probe
`M_MultiDim_FixedArray` (`#[ignore]`-flagged).

**Workaround**: probe ignored. Real code can use nested
single-dim arrays (`ARRAY M OF ARRAY N OF T`) instead, which
works because each dimension is its own GEP step.

**Why deferred**: the comma-syntax `arr[i, j]` lowers to
something that loads the inner `[N x T]` row as a value and then
tries to use it as an index. The fix is to chain GEPs instead of
materialising the inner row.

**Closing it**: in `newcp-ir`'s multi-index lowering, emit a
sequence of `IndexGep` instructions (one per dimension) instead
of a single GEP that drops the trailing index. Un-ignore the
probe to confirm; expected packed value 250.

### 22. Method dispatch on a call-result receiver reads wild memory

**Where**: receiver lowering in `newcp-ir/src/lower.rs`. Surfaced
by matrix probe `M_Method_On_Function_Result`
(`#[ignore]`-flagged).

**Workaround**: probe shipped ignored. Real code can assign the
call result to a local first and call through the local — that
path uses a designator-based receiver, which works.

**Why deferred**: the plain-record-dispatch refactor (earlier in
the test-matrix landing) lowers receivers via `designator_addr`.
A call-as-prefix (`Make(99).Get()`) produces a Temp IrValue
rather than a designator, so the receiver lookup reads the wrong
slot and the body dereferences an uninitialised pointer
(observed: `1881534181456` instead of `99`).

**Closing it**: in the bound-proc-call lowering, when the prefix
expression is itself a procedure call (rather than a designator),
materialise its result into a synthetic Temp slot and use that
Temp as the receiver pointer. Un-ignore the probe to confirm.

### 23. Sema mis-types receiver as the underlying record when the method's return type is the receiver's pointer alias

**Where**: sema's return-type checking. Surfaced by matrix probe
`M_Method_Returns_Pointer` (`#[ignore]`-flagged).

**Workaround**: probe shipped ignored. A method that returns
`Box` (pointer alias) can be rewritten to return `BoxDesc` (the
record) but that loses the BlackBox-faithful builder-pattern
idiom; or the caller can wrap in `(b: Box) Method (): BoxDesc`
and have callers narrow back. Neither is great.

**Why deferred**: the receiver-canonicalisation work elsewhere in
sema already chases pointer aliases to their record names so
method dispatch lines up. The return-type check needs the
mirror-image canonicalisation: when comparing `RETURN b` (where
`b` is a pointer-aliased receiver) against the declared return
type `Box`, sema must recognise that `b`'s type and `Box` agree.
Currently it reports `expected Box, found BoxDesc`.

**Closing it**: in sema's `check_return_type` / equivalent, when
the actual type is a record and the expected type is a pointer
alias to that same record, allow the implicit narrowing (or
better: canonicalise both sides to the underlying record name
before comparing). Un-ignore the probe to confirm.

### 24. SHORTREAL mixed with REAL operand produces wild result

**Where**: codegen / IR lowering for mixed-precision float
arithmetic. Surfaced by matrix probe
`M_Expr_SHORTREAL_Arithmetic` (`#[ignore]`-flagged).

**Workaround**: probe ignored. Real code can stick to REAL
throughout, which works.

**Why deferred**: `SHORT(3.0) * SHORT(2.5) * 2.4` evaluates to
2097152 instead of 18.0.  Either the SHORTREAL → REAL promotion
on the third operand is wrong, or ENTIER on the REAL result
truncates the wrong bits, or the IR is treating the f32 as f64
without converting first.

**Closing it**: trace the float-width-promotion lowering in
`newcp-ir/src/lower.rs::lower_binary` (the existing
`is_float_ir`-driven branch handles int→float but may be missing
SHORTREAL→REAL). Un-ignore the probe to confirm; expected 18.

### 25. Calling a procedure-typed *record field* mis-routes through a direct call

**Where**: call-site lowering in `newcp-ir`. Surfaced by matrix
probe `M_Type_ProcedureField_InRecord` (`#[ignore]`-flagged).

**Workaround**: probe ignored. Real code can copy the field into
a local procedure-typed var first and call through the local
(that path works — see `M_ProcType_IndirectCall`).

**Why deferred**: `d.f(7)` where `d.f` is a record field of
procedure type tries to emit `call DispatcherDesc_f` (a mangled
method-style name) instead of loading the field and emitting an
indirect call through the loaded function pointer.

**Closing it**: in `lower_bound_proc_call_expr` (or its sibling
that handles `obj.field` callables), recognise when the prefix
designator resolves to a procedure-typed field — emit a Load of
the field then an indirect Call, the same path a procedure-typed
local variable uses. Un-ignore the probe to confirm; expected 49.

### 26. Sema rejects relational `<` / `<=` / `>` / `>=` on ARRAY OF CHAR

**Where**: sema's relational-operator type table. Surfaced by
matrix probe `M_Expr_String_Compare_Mixed` (`#[ignore]`-flagged).

**Workaround**: probe ignored. Real code that wants
lexicographic ordering can call a runtime helper directly.

**Why deferred**: CP §8.2.5 defines `<` etc on `ARRAY OF CHAR`
as character-by-character lexicographic. The string-equality
helper (`__newcp_string_eq_char`) already exists; we'd need
companion ordering helpers and sema acceptance of the operand
pair.

**Closing it**: add ARRAY-OF-CHAR-vs-ARRAY-OF-CHAR rows to the
relational-operator sema table for `<`, `<=`, `>`, `>=`; emit
calls to new runtime helpers (mirroring the `=` / `#` path).
Un-ignore the probe to confirm; expected 111.

### 27. INC on BYTE doesn't update the variable

**Where**: codegen of `INC(b, delta)` when b is BYTE.  Surfaced
by matrix probe `M_Expr_INC_OnByte` (`#[ignore]`-flagged).

**Workaround**: probe ignored. Real code can read into INTEGER,
compute, narrow with SHORT-chain.

**Why deferred**: INC(b, 50) leaves b unchanged (observed:
returns 100 instead of 150).  Likely an integer-width mismatch
in the IR store path — the delta is INTEGER (i64) but the slot
is BYTE (i8); something in the store either drops to the wrong
slot or stores into a temp that's never read back.

**Closing it**: trace INC's IR lowering for BYTE/SHORTINT/INTSHORT
targets — verify the load+add+store chain types align and the
final store lands in the named slot.  Un-ignore the probe;
expected 150.

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
