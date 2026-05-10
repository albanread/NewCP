# Bug: `SHORT(I64)` always truncates, ignoring NewCP's `LONGINT == INTEGER` width collapse

## Summary

NewCP maps both `LONGINT` and `INTEGER` to `i64` at the IR/LLVM level,
but `BuiltinProc::Short` lowering in `newcp-ir` unconditionally narrows
any `i64` input to `i32`. Source written for BlackBox semantics
(`SHORT(longint_expr)` → `INTEGER`, intended as a width-changing cast)
silently truncates a value that should have remained 64-bit. The
mismatch only becomes visible when LLVM rejects the resulting IR with
`Call parameter type does not match function signature` at a call site
that expects `INTEGER` (`i64`) but receives the truncated `i32`.

## Reproducer

[Mod/Integers.cp](../Mod/Integers.cp) `Entier`, `Power`:

```cp
y := New(mL + SHORT(ENTIER(Math.Ln(2) * ex / Math.Ln(B)) + 1));
```

`ENTIER` returns `LONGINT` (`i64`). `+ 1` keeps it `LONGINT`. `SHORT`
narrows `LONGINT → INTEGER` (semantically). `New (nofDigits: Index)`
takes `Index = INTEGER` (`i64`).

Compile:

```
cargo run --quiet --bin newcp-driver -- dump-llvm --opt none Mod/Integers.cp
```

Verifier output:

```
Call parameter type does not match function signature!
  %cast.trunc = trunc i64 %iadd13 to i32
 i64  %t61 = call ptr @New(i32 %cast.trunc)
```

The `cast.trunc` came from the `SHORT` arm; the call site expects `i64`.

## Root cause

[`src/newcp-ir/src/lower.rs`](../src/newcp-ir/src/lower.rs), `lower_builtin_expr` SHORT arm:

```rust
"SHORT" => {
    let x = self.lower_expr(args.first()?);
    let from_ty = x.ty();
    let to_ty = match &from_ty {
        IrType::Char => IrType::ShortChar,
        IrType::I64 => IrType::I32,    // <-- ambiguous
        IrType::F64 => IrType::F32,
        IrType::I32 => IrType::I16,
        other => other.clone(),
    };
    ...
}
```

`I64 → I32` is correct for `INTEGER → INTSHORT`, wrong for
`LONGINT → INTEGER` (which should be a no-op at IR level because both
land on `i64`). The lowerer cannot distinguish the two because it
knows only `IrType`, not the semantic type that produced it.

## Why it's hard to fix in isolation

Sema's `BuiltinType` distinguishes `LongInt` and `Integer`; IR's
`IrType` collapses both to `I64`. The narrowing chain in
[`newcp-sema/src/lib.rs`](../src/newcp-sema/src/lib.rs)
(`short_result_type`) has 5 levels (`LongInt → Integer → IntShort →
ShortInt → Byte`); the IR width chain has 4 (`I64 → I32 → I16 → I8`).
There's no IR-level fact that tells `lower_builtin_expr` whether a
given `i64` value was sema-tagged as `LongInt` (this `SHORT` should be
a no-op) or `Integer` (this `SHORT` should truncate to `i32`).

## Fix sketch

Plumb a sema-type-of-expression accessor into the IR lowerer. Two
viable shapes:

1. **Re-export sema inference.** Make `Analyzer::infer_expr_type` (and
   the helpers it depends on — `infer_designator_type`,
   `lookup_symbol_type`, etc.) callable as free functions taking
   `(expr, local_symbols, module_symbols, scope_type_names) ->
   Option<SemanticType>`. Add a thin wrapper on `LowerCtx` that
   forwards. In the SHORT arm, call `sema_type_of_expr(arg)` and
   choose the cast based on the *semantic* narrowing step, not the IR
   width.

2. **Tag IR values with their semantic type.** Add an optional
   `sema_ty: Option<BuiltinType>` to `IrValue` (or a side-table keyed
   by `TempId`). Populate at lowering sites that produce typed values.
   SHORT consults the tag.

Option 1 is less invasive and reuses sema's existing logic. Option 2
makes the information available to all future arithmetic-promotion
checks (LONG, ABS, MAX/MIN, SET ops) without re-inferring.

Once the SHORT arm is sema-aware, also audit:

- `LONG` (mirror issue: widening from `Integer → LongInt` is a no-op
  at IR level, but `LONG(I32) → I64` is real).
- `MAX`/`MIN` two-argument sign-bit-mask lowering uses `x.ty()` to
  pick `bits`; OK today because both ops are `i64`-or-narrower, but
  worth a second look once semantic typing is available.

## Affected ports

- [Mod/Integers.cp](../Mod/Integers.cp) — multiple call sites in
  `Entier`, `Power`, and `Long` use `SHORT(LONGINT)` to convert
  back to `INTEGER`. Source-level workaround (deleting the SHORTs)
  would diverge from the BlackBox original; not done.

## Test plan when fixing

1. Add a focused IR test: lower `SHORT(LONG(x))` where `x: INTEGER`
   and check that no `Cast` instruction is emitted (round-trip is a
   no-op at IR level when both ends are `i64`).
2. Re-run `cargo run --bin newcp-driver -- dump-llvm Mod/Integers.cp`
   and confirm verifier-clean output.
3. Add `Mod/Tests/IntegersSmoke.cp` covering `Sum`, `Difference`,
   `Product`, `Quotient`, `Power`, `ConvertFromString` /
   `ConvertToString`. Wire into `newcp-tests`.
4. Run the full `cargo test -p newcp-tests` suite — current baseline
   176/176.

## Cross-references

- [docs/deferred_fixes.md](deferred_fixes.md) items 12 & 13.
- Checkpoint commit `cd78665` landed the rest of the Integers
  prerequisites (sema alias resolution, OUT-pointer-array indexing,
  `MAX`/`MIN` with user aliases, nested-proc symbol shadowing).
