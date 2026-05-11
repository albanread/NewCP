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

### 14. ~~`NEW(record_field_pointer)` — IR can't resolve destination record type~~ — FIXED

**Status**: closed. The NEW lowering in `newcp-ir` now uses
`designator_ir_type` (which walks selectors) to resolve the
destination type, with `base_symbol_ir_type` as a fallback for
bare locals. Handles record-field pointers, array-element
pointers, and chains thereof.

**Regression coverage**: matrix probes
`M_Method_On_RecordField`, `M_Type_Pointer_To_Pointer`,
`M_Method_On_ArrayElement`, `M_Record_With_Pointer_Field` all
un-ignored.

### 16. ~~`IS` test against an uninstantiated record type crashes~~ — FIXED

**Status**: closed. The IR's IS-test lowering was using the
pointer-alias name (`Box`) as the target type name, but every
emitted `<Type>.desc` global is keyed under the *record* name
(`BoxDesc.desc`).  The mismatch left the LLVM call site pointing
at an unresolved external `@Box.desc` symbol; when the loader
mapped it to a wild address, the runtime type-test segfaulted.

New helper `LowerCtx::canonical_named_record` (same-module +
imported, with the existing import cache) strips one level of
pointer-alias indirection before the IS-test target is recorded.
The runtime `__newcp_type_test` itself already handled NIL
targets correctly; the bug was purely the name mismatch on the
LLVM side.

**Regression coverage**: four matrix probes un-ignored —
`M_AnyPtr_IS_Test`, `M_Expr_Pointer_IS_Test`,
`M_Type_ANYREC_Param`, `M_Stmt_IS_Inside_WITH`.

### 17. ~~`IN p: PointerAlias` dereference crashes at runtime~~ — FIXED

**Status**: closed. The root cause was the call site, not the
callee: `lower_args_with_signature` only passed slot addresses
for VAR/OUT params and for IN open-arrays.  For `IN p: Box`
(non-open-array IN), it fell through to a path that loaded the
pointer value and passed it directly, so the callee got `p`'s
value where it expected `&p`. The callee then dereferenced what
it thought was a slot, treating the pointed-to byte as a slot
address itself → segfault.

Fix: extend the address-by-default branch to cover `Some(ParamMode::In)`
for non-open-array params so the call site uniformly passes
slot addresses for IN/VAR/OUT, matching the callee's `Ref(T)`
formal layout.

**Regression coverage**: matrix probe `M_Param_IN_Pointer_Deref`
un-ignored.

### 18. ~~Indirect call through a procedure-typed parameter mis-types its args~~ — FIXED

**Status**: closed. Two coordinated fixes:
- Sema (`apply_selector_type` and the
  `validate_call_arguments`-driven path): unwrap Named alias
  chains on `base` so a parameter declared with a procedure-type
  alias (`f: Unary`) is recognised as callable instead of being
  mis-interpreted as a type guard.
- IR (`lower_designator`'s indirect-call path): accept
  `Selector::AmbiguousParen(qual)` as a single-arg call form,
  not just `Selector::Call(args)`.  The AmbiguousParen tentative
  form survives when the parser couldn't tell call from
  type-guard syntactically; once the IR knows the base is a
  procedure value, the disambiguation is unambiguous.

**Regression coverage**: matrix probe `M_ProcType_Param_Callback`
un-ignored.

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

### 20. ~~Sema rejects subset/superset (`<=` / `>=`) on SET operands~~ — FIXED

**Status**: closed. `are_ordered_relation_compatible` in
`newcp-sema` now accepts (SET, SET); `lower_binary` in `newcp-ir`
emits the subset / superset / strict-variant lowerings via the
bitwise identity `s1 <= s2 iff s1 * s2 = s1`.

**Regression coverage**: matrix probe `M_Expr_SET_Equality` covers
the operator pair on small overlapping sets.

### 21. ~~Multi-dimensional fixed-array indexing crashes codegen~~ — FIXED

**Status**: closed. `designator_addr` was already emitting one
`IndexGep` per index in a multi-index selector, but
`designator_ir_type` only stripped a single Array/Ptr wrapper
per `Selector::Index(_)` arm regardless of how many indices
the selector held. The mismatched final type meant the eventual
`Load` was typed `[N x T]` (a whole row) rather than the scalar
element, which is what triggered the `Found ArrayValue but
expected the IntValue variant` panic downstream when the loaded
row reached a binop. Fix: in
`newcp-ir/src/lower.rs::designator_ir_type`, when the selector
is `Selector::Index(indices)`, peel one Array/Ptr/Named
wrapper per index — mirroring the IR emission walk in
`designator_addr`.

**Regression coverage**: matrix probe `M_MultiDim_FixedArray`
un-ignored; computes the packed value 250 from a 3×3 grid.

### 22. ~~Method dispatch on a call-result receiver reads wild memory~~ — FIXED

**Status**: closed. Two-part fix in `lower_bound_proc_call_expr`:
- When the prefix designator ends in a Call/AmbiguousParen
  selector, lower the prefix via `lower_expr` (evaluates the
  call) instead of `designator_addr` (which has no addressable
  storage for a call result).
- `designator_ir_type` gained a `Selector::Call` arm that
  returns the procedure's declared result type, so the receiver-
  type lookup succeeds for chained-call designators.

**Regression coverage**: matrix probe `M_Method_On_Function_Result`
un-ignored.

### 23. ~~Sema mis-types receiver as the underlying record when the method's return type is the receiver's pointer alias~~ — FIXED

**Status**: closed. New helper
`return_is_receiver_pointer_round_trip` in `newcp-sema`'s Return-
statement check allows `RETURN <receiver>` when the receiver
designator names a `SymbolKind::Receiver` formal and the
expected return type's pointer target unwraps (through any
alias chain) to the same Record as the receiver's canonicalised
declared type.

**Regression coverage**: matrix probe `M_Method_Returns_Pointer`
exercises the BlackBox-style builder idiom
`(b: Box) WithValue (n: INTEGER): Box; ... RETURN b`.

### 24. ~~SHORTREAL mixed with REAL operand produces wild result~~ — FIXED

**Status**: closed. `lower_binary` only promoted INT→FLOAT for
mixed-type expressions and didn't widen SHORTREAL (f32) when the
other operand was REAL (f64). The resulting `BinOp` had
mismatched IR widths; LLVM emit interpreted the f32 bit pattern
as the low 32 bits of an f64 (the high half coming from whatever
register state was around), producing magnitudes like 2097152
instead of 18.0. Fix: when both operands are floats but widths
differ, emit a `Cast` to widen the narrower side to f64 before
the BinOp.

**Regression coverage**: matrix probe
`M_Expr_SHORTREAL_Arithmetic` un-ignored; `ENTIER(SHORT(3.0) *
SHORT(2.5) * 2.4)` now produces 18.

### 25. ~~Calling a procedure-typed *record field* mis-routes through a direct call~~ — FIXED

**Status**: closed. `lower_bound_proc_call_expr` now checks
whether the "method name" actually names a procedure-typed
**field** of the receiver record (after unwrapping any Named
alias chain to land on the underlying `Procedure(...)`). When it
does, the lowering re-emits the prefix designator + field
selector as a value expression (yielding the function pointer)
and emits an indirect `Instr::Call` through it. Methods stay
on the existing dispatch path.

**Regression coverage**: matrix probe
`M_Type_ProcedureField_InRecord` un-ignored.

### 26. ~~Sema rejects relational `<` / `<=` / `>` / `>=` on ARRAY OF CHAR~~ — FIXED

**Status**: closed. `are_ordered_relation_compatible` now accepts
the `is_string_like_type` pair (string builtins + ARRAY OF
CHAR/SHORTCHAR).  New runtime helpers `__newcp_string_cmp_char`
and `__newcp_string_cmp_shortchar` return -1/0/1; the
`Instr::StringCompare` IR variant grew an op enum so the same
node carries Eq/Ne (existing eq helper) and Lt/Le/Gt/Ge (new
cmp helper, chained with an integer compare against 0).

**Regression coverage**: matrix probe `M_Expr_String_Compare_Mixed`
exercises `<`, `<=`, `>` between two `ARRAY 8 OF CHAR` variables.

### 27. ~~INC on BYTE doesn't update the variable~~ — FIXED

**Status**: closed. `lower_inc_dec_statement` was passing the
delta as its native lower_expr width (INTEGER → i64) to a BinOp
whose result type was the target's slot type (BYTE → i8).  The
width mismatch produced a BinOp the LLVM emit dropped on the
floor.  Fix: emit an explicit `Cast` to narrow / widen the delta
to the target's slot type before the BinOp.

**Regression coverage**: matrix probe `M_Expr_INC_OnByte`
un-ignored.

### 28. SET constant membership wrong value

**Where**: SET literal / constant membership lowering. Surfaced
by matrix probe `M_Expr_SET_Constant_Membership`
(`#[ignore]`-flagged).

**Status**: filed for investigation. Probe returns 101 vs the
expected packed value. Either constant SET folding is buggy or
`IN` on a constant-LHS short-circuits incorrectly.

### 29. NEW on `POINTER TO ARRAY n OF T` (fixed array) fails

**Where**: IR's `Instr::New` resolution. Surfaced by matrix probes
`M_Type_PointerTo_FixedArray` and
`M_Type_PointerTo_FixedArray_AsField` (both ignored).

**Status**: filed. `Instr::New: unknown record type [N x T]`.  My
recent #14 fix to walk the designator's IR type uncovered this
case — when the target's underlying type is a fixed-array
(`[N x T]`) rather than a record, the NEW lowering needs a
different allocator path (basically just heap-alloc N*sizeof(T)
bytes).

### 30. Module-level VAR with INLINE record type fails codegen

**Where**: LLVM emit for assignment / comparison involving
inline-record types. Surfaced by
`M_Module_VAR_Record_DefaultZero`.

**Status**: filed. `non-equality pointer comparison Add` — the
inline-record slot's address arithmetic is mis-routed. Real code
uses named TYPE records and works; the inline form is an unusual
but legal CP idiom.

### 31. Type-guard designator as LHS of assignment rejected

**Where**: sema's `validate_assignment_target`. Surfaced by
`M_Expr_TypeGuard_AsLHS_Designator`.

**Status**: filed. `p(Sub).field := value` reports
`assignment target is not assignable`. The type guard yields
what should be an addressable narrowed view; sema is excluding
it from valid LHS forms. Workaround: assign through an
intermediate typed variable.

### 32. SYSTEM.MOVE between arrays doesn't actually copy

**Where**: runtime / IR lowering of SYSTEM.MOVE. Surfaced by
`M_SYSTEM_MOVE_BetweenArrays`.

**Status**: filed. dst stays zero (observed sum=0 instead of 10
after MOVE'ing 4 bytes between two arrays). Either the
intrinsic dispatches to a no-op stub or the address arguments
are being misread.

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
