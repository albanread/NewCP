# Bug report: two cross-module obstacles encountered during the Fonts port

Both surfaced while bringing up the BlackBox-faithful `Fonts` /
`HostFontsSys` / `HostFonts` stack on top of the new
`iGui.MeasureFont` / `iGui.MeasureString` service. Workarounds are in
place so the demo (`Mod/FontProbe.cp`) passes end-to-end, but neither
workaround scales beyond a single Host module — every subsequent BlackBox
subsystem we port (`Ports`, `Stores`, `Models`, `Views`) will hit the
same walls the moment it tries to faithfully recreate BlackBox's
abstract-base / concrete-Host split.

---

## Issue 1 — Sema infinite recursion on local TYPE name colliding with imported TYPE name

Already documented in detail in
[`bug_report_sema_name_collision.md`](bug_report_sema_name_collision.md).
Brief recap so the two findings stay together:

- Declaring `TYPE T = RECORD (Imported.T) ... END` (or any local
  type whose simple name shadows an imported type) sends `newcp-sema`
  into unbounded recursion.
- Triggered cleanly with a 6+9-line two-module repro:

  ```cp
  MODULE TestBase;
    TYPE  DirectoryDesc* = ABSTRACT RECORD END;
          Directory*     = POINTER TO DirectoryDesc;
  END TestBase.
  ```

  ```cp
  MODULE TestExtend;
    IMPORT TestBase;
    TYPE  DirectoryDesc* = RECORD (TestBase.DirectoryDesc) END;
          Directory*     = POINTER TO DirectoryDesc;
  END TestExtend.
  ```

  `check-mod TestExtend` overflows the stack. Renaming the local
  `DirectoryDesc` / `Directory` to anything non-colliding fixes it.

- Workaround currently shipped in [`Mod/HostFonts.cp`](../Mod/HostFonts.cp):
  every local extension type uses a private lowercase name
  (`fontImpl`, `dirImpl`, `fontImplPtr`, `dirImplPtr`) instead of the
  BlackBox-faithful `FontDesc` / `Font` / `DirectoryDesc` / `Directory`.

- Why the workaround is uncomfortable: BlackBox modules consistently
  reuse the parent's name in the child (e.g. `HostPorts.Port` extends
  `Ports.Port`, `HostFonts.Directory` extends `Fonts.Directory`).
  Forcing every Host module to rename its public types away from its
  parent's public types makes the port visibly diverge from its
  BlackBox source. We need this fixed before porting `HostPorts`,
  `HostFiles`, `HostFonts`, etc. lands more renames.

---

## Issue 2 — JIT vtable leaves a slot empty for an inherited concrete method whose body lives in another module

### Symptom

```
$ newcp-driver run-igui FontProbe.Run
...
FontProbe: measuring default font
newcp trap: virtual call to inherited method whose body lives in another
module — vtable slot was left unimplemented at JIT time
(see docs/files_module_investigation.md, item 2)
```

The runtime trap message is helpfully self-describing — it points
straight at [`files_module_investigation.md`](files_module_investigation.md)
item 2, which already notes the limitation. This report is to capture a
fresh user-visible reproducer outside the Files context, and to flag
that it now blocks the BlackBox-faithful Fonts/HostFonts port the same
way it blocks Files.

### Reproducer

`Mod/Fonts.cp` declares an abstract `FontDesc` plus one *concrete*
type-bound proc `Init` that just stores the constructor arguments
into the record fields:

```cp
MODULE Fonts;
TYPE
  FontDesc* = ABSTRACT RECORD
    typeface-: Typeface; size-: INTEGER; style-: SET; weight-: INTEGER
  END;
  Font* = POINTER TO FontDesc;

PROCEDURE (f: FontDesc) Init*
  (typeface: Typeface; size: INTEGER; style: SET; weight: INTEGER), NEW;
BEGIN
  ASSERT(f.size = 0, 20); ASSERT(size # 0, 21);
  f.typeface := typeface; f.size := size;
  f.style := style; f.weight := weight
END Init;
(* ... abstract methods ... *)
END Fonts.
```

`Mod/HostFonts.cp` extends `FontDesc` and overrides the abstract
methods. It does *not* override `Init` — it inherits it. Inside its
`Directory.This` factory it calls the inherited method:

```cp
PROCEDURE (d: dirImpl) This*
  (typeface: Fonts.Typeface; size: INTEGER;
   style: SET; weight: INTEGER): Fonts.Font;
  VAR f: fontImplPtr;
BEGIN
  NEW(f);
  f.Init(typeface, size, style, weight);   (* <-- traps here *)
  RETURN f
END This;
```

`f.Init(...)` dispatches via the vtable. The slot for `Init` in
`HostFonts.fontImpl`'s vtable is supposed to point at the body that
lives in `Fonts` module. The single-module JIT vtable patcher only
fills slots for bodies inside *this* module's compilation unit, so the
slot stays as the zero-initialiser, and the runtime traps on dispatch.

Notably:

- *Overridden* methods work — `Default`, `GetBounds`, `StringWidth`,
  `SStringWidth`, `IsAlien` all dispatch correctly because their bodies
  live in `HostFonts`, the same module whose vtable global is being
  patched.
- *Abstract* methods that aren't overridden anywhere are obviously
  expected to trap.
- Only *concrete inherited* methods (override-not-needed, body lives
  upstairs in the parent module) hit this gap.

### Workaround currently shipped

Inline the parent's Init body inside the child's `This` factory:

```cp
PROCEDURE (d: dirImpl) This*
  (typeface: Fonts.Typeface; size: INTEGER;
   style: SET; weight: INTEGER): Fonts.Font;
  VAR f: fontImplPtr;
BEGIN
  NEW(f);
  IF typeface = Fonts.default THEN typeface := defaultTypeface END;
  (* Initialize fields directly rather than calling the inherited
     Fonts.FontDesc.Init: the JIT can't currently patch a vtable slot
     that points across module boundaries. The asserts in the original
     Init are duplicated here to preserve the invariant. *)
  ASSERT(f.size = 0, 20);
  ASSERT(size # 0, 21);
  f.typeface := typeface; f.size := size;
  f.style := style; f.weight := weight;
  RETURN f
END This;
```

After this change `FontProbe.Run` produces sensible metrics:

```
default font cell (BB sub-mm units):
  ascent  = 137046       (~14.4 DIPs)
  descent = 31874        (~3.3 DIPs)
  width   = 114040       (~12.0 DIPs — Cascadia Mono M-advance)
```

### Why this matters beyond Fonts

BlackBox uses `(record) MethodName, NEW` (concrete on the parent) and
inherits-without-override extensively for "set fields, perform
invariants, store the constructor's input" idioms. Examples just from
the corpus's System layer:

- `Files.FileDesc.InitType` (concrete, inherited by `HostFiles.File`)
- `Stores.StoreDesc.Init` (concrete, inherited by every Store subclass)
- `Models.ModelDesc.Init` (likewise)
- `Views.ViewDesc.Init` (likewise)
- `Controllers.ControllerDesc.Init` (likewise)
- `Ports.Port.Init` (concrete, inherited by `HostPorts.Port`)

Every one of these is currently a trap waiting to fire the moment we
port the corresponding `HostXxx`. We can keep inlining the parent's
body at every call site, but that's a recipe for invariants drifting
out of sync between parent and child as the modules evolve.

### Suggested fixes (ranked by ambition)

1. **Cross-module vtable patching** at JIT time — when finalizing
   module B's vtable for a record `B.T` that extends `A.T`, walk the
   inheritance chain into `A`, look up each concrete inherited method's
   address from `A`'s already-loaded JIT image, and write that address
   into the corresponding slot of B's vtable. This is the right fix.
2. **Static (non-virtual) call lowering for inherited concrete
   methods** — at the IR level, recognise that
   `Fonts.FontDesc.Init` has a known concrete body and lower
   `f.Init(...)` as a direct call to that body's symbol, bypassing the
   vtable entirely. Misses dynamic-dispatch semantics if a future
   subclass were to override `Init`, but that's not a real concern for
   constructor-style methods.
3. **Surrogate/trampoline body** — when emitting B's module image,
   emit a thunk for each inherited concrete method that simply forwards
   to A's body via a runtime symbol lookup. The B-side patcher fills B's
   vtable slot with the trampoline; the trampoline fixes itself up on
   first call. More plumbing than (1) but doesn't require the JIT
   patcher to know about other modules.

---

## Combined effect on the porting plan

`Mod/Fonts.cp` + `Mod/HostFontsSys.cp` + `Mod/HostFonts.cp` + the
demo `Mod/FontProbe.cp` are committed and run, but with both
workarounds applied. To proceed cleanly with `HostPorts` (the
infrastructural prerequisite for `ObxViews0`) we'd want both issues
addressed — the OOP layering pattern is going to repeat ~15 times
across the System tier of the BlackBox corpus.
