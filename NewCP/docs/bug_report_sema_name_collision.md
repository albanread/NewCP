# Bug: Sema infinite-recurses on local TYPE name colliding with imported TYPE name

## Summary

When a module declares a local `TYPE T = RECORD (Imported.T) ... END`
— same simple name `T` as the imported base — `newcp-sema` enters
unbounded recursion (eventually stack-overflows or hangs the driver).
The collision happens whether or not the imported type is actually
extended; the trigger is just the name overlap during type resolution.

## Reproducer

The case discovered in the wild was [Mod/HostFonts.cp](../Mod/HostFonts.cp).
Originally the module wanted to declare:

```cp
MODULE HostFonts;
IMPORT Fonts;
TYPE
  FontDesc      = RECORD (Fonts.FontDesc) END;     (* SHADOWS Fonts.FontDesc *)
  Font          = POINTER TO FontDesc;
  DirectoryDesc = RECORD (Fonts.DirectoryDesc) END;
  Directory     = POINTER TO DirectoryDesc;
...
```

Running `cargo run --bin newcp-driver -- check-mod Mod/HostFonts.cp`
on this form blows the stack inside sema. The local `FontDesc`'s
declared base resolves through the type table by simple name, finds
the local `FontDesc` again, recurses.

The shipped workaround is to rename every local type to a non-colliding
name:

```cp
TYPE
  fontImpl     = RECORD (Fonts.FontDesc) END;
  fontImplPtr  = POINTER TO fontImpl;
  dirImpl      = RECORD (Fonts.DirectoryDesc) END;
  dirImplPtr   = POINTER TO dirImpl;
```

This works because `HostFonts` has no public API — it just registers
itself with `Fonts` at startup via `Fonts.SetDir`. Modules that do
need to re-export the same simple name (a legitimate CP idiom for
host-specialised implementations of an abstract interface) don't have
this escape hatch.

## Question to settle

**Is shadowing an imported type with a local declaration of the same
simple name actually valid Component Pascal?**

- BlackBox compiles modules like `HostFiles`, `HostDates`, `WinFonts`
  that re-use the abstract `Files` / `Dates` / `Fonts` simple names
  freely; the local declaration shadows the import inside the
  module's own body and clients see only the local one.
- In CP scoping rules, an `IMPORT M` introduces `M` as the *only*
  qualifier into scope; simple names from `M` are reachable only as
  `M.X`. A local `TYPE X` therefore doesn't actually collide with
  `M.X` at the language level — they live in different namespaces
  (qualified vs unqualified). So the shadowing is *legal* and
  BlackBox-idiomatic.

If we agree the answer is **valid**, sema must resolve the local
declaration to itself for unqualified references and to the imported
declaration for qualified `M.X` references. Today the resolver loses
the qualification on the way through some recursive path and re-finds
the local symbol.

If for some defensible reason we want to **forbid** the pattern,
sema must reject it with a clean diagnostic at TYPE-declaration time
("local type `X` collides with imported `M.X`; rename one") instead
of stack-overflowing during inheritance walk.

Either outcome is acceptable. The current behaviour — silent infinite
recursion at check-mod time — is not.

## Likely fault sites

`newcp-sema/src/lib.rs` has several type-walk paths, some of which
have cycle protection (`seen_named: HashSet<String>` in
`resolve_named_type_alias`) and some of which don't. The hang lives
in one of the unguarded walks. Candidate suspects:

- `find_inherited_method` walks the inheritance chain by calling
  `record_type_info(&base_name)` repeatedly with no `seen` set. If
  `record_type_info` resolves a base name to a type whose own base
  recurses to itself, the loop never terminates.
- `record_inherits_method` and `find_record_decl` similarly walk by
  name without cycle protection.
- `Analyzer::resolve_alias_to_builtin_target` (added in checkpoint
  `cd78665`) caps at 16 iterations and is therefore safe; same shape
  could be applied elsewhere.

A small printf/`eprintln!` at each by-name-base-walk entry would
pinpoint which loop is the offender — recommended first step.

## Fix sketches

**If shadowing is to be allowed (preferred):**

1. At resolution time, distinguish "local type T" from "imported type
   `M.T`" everywhere. Always carry the qualifier through. Where
   resolution walks an inheritance chain, look the base up by its
   *fully qualified* name, not its simple name.
2. Add cycle protection (`HashSet<(Option<String>, String)>` of
   already-visited `(module, name)` pairs) to every recursive
   type-walk path in sema as a defensive net. Even the "shouldn't
   happen" cases shouldn't blow the stack.

**If shadowing is to be forbidden:**

1. In `collect_module_symbols` (or wherever TYPE decls are recorded),
   detect simple-name collision between a new TYPE and any
   already-imported TYPE symbol. Emit a diagnostic of the form
   `local type 'FontDesc' shadows imported 'Fonts.FontDesc' — sema
   does not yet support same-name extension; rename the local type`.
2. Apply the same cycle-protection from the previous option as a
   defensive net so future regressions surface as clean errors.

## Test plan when fixing

1. Add `Mod/Tests/SemaShadowsImport.cp`:
   ```cp
   MODULE SemaShadowsImport;
   IMPORT Fonts;
   TYPE
     FontDesc* = RECORD (Fonts.FontDesc) END;
     Font*     = POINTER TO FontDesc;
   END SemaShadowsImport.
   ```
   With "allow" semantics: `check-mod` succeeds, `dump-sema` shows
   `Fonts.FontDesc` as the base of the local `FontDesc`. With
   "forbid" semantics: `check-mod` produces the rename diagnostic
   and exits cleanly.
2. Add an explicit cycle-detection test:
   ```cp
   TYPE
     A = RECORD (B) END;
     B = RECORD (A) END;
   ```
   This is malformed source; sema should diagnose the cycle, never
   stack-overflow.
3. Restore the original `HostFonts` declarations (rename `fontImpl`
   back to `FontDesc`, etc.) and confirm it works.
4. Existing baseline: `cargo test -p newcp-tests` passing 176/176.

## Cross-references

- Workaround in [Mod/HostFonts.cp](../Mod/HostFonts.cp) — local
  types renamed to `fontImpl` / `dirImpl` to dodge the collision
  (with a comment explaining why).
- Related but different issue: [bug_report_short.md](bug_report_short.md)
  (sema/IR semantic-vs-IR-width mismatch).
