MODULE KernelProbe;
(* Smoke probe for the Kernel runtime binding.

   Allocates a record, asks Kernel for its TypeDesc, walks the
   inheritance chain, and pokes the size / level / base accessors.
   The test harness calls the procs directly via run_function so
   each result is verifiable in isolation.

   This is the "first slice" of the Kernel surface — Time, type
   reflection, NewObj. Trap cleaners and the event loop come later. *)

IMPORT Kernel;

TYPE
  WidgetDesc* = RECORD
    value*: INTEGER;
    flag*:  BOOLEAN
  END;
  Widget*     = POINTER TO WidgetDesc;

  GadgetDesc* = RECORD (WidgetDesc) extra*: INTEGER END;
  Gadget*     = POINTER TO GadgetDesc;

(* Methods so the compiler emits TypeDescs for both records. *)

PROCEDURE (w: WidgetDesc) Touch*(), NEW;
BEGIN END Touch;

PROCEDURE (g: GadgetDesc) TouchExtra*(), NEW;
BEGIN END TouchExtra;

(* -- Probes ------------------------------------------------------------- *)

(** TypeOf on an allocated Widget returns its declared TypeDesc; we
    can round-trip that handle through SizeOf and LevelOf. Returns 1
    if every check passes, 0 otherwise. *)
PROCEDURE WidgetReflection*(): INTEGER;
  VAR w: Widget; t: Kernel.Type;
BEGIN
  NEW(w);
  t := Kernel.TypeOf(w);
  IF t = NIL THEN RETURN 0 END;
  IF Kernel.LevelOf(t) # 0 THEN RETURN 0 END;       (* Widget is a root *)
  IF Kernel.SizeOf(t) <= 0 THEN RETURN 0 END;       (* non-empty payload *)
  IF Kernel.BaseOf(t) # NIL THEN RETURN 0 END;      (* no base type *)
  RETURN 1
END WidgetReflection;

(** Same for an extension type — Gadget extends Widget, LevelOf = 1
    and BaseOf chains back to Widget's TypeDesc. *)
PROCEDURE GadgetReflection*(): INTEGER;
  VAR g: Gadget; gt, wt: Kernel.Type;
  VAR w: Widget;
BEGIN
  NEW(g);
  NEW(w);
  gt := Kernel.TypeOf(g);
  wt := Kernel.TypeOf(w);
  IF (gt = NIL) OR (wt = NIL) THEN RETURN 0 END;
  IF Kernel.LevelOf(gt) # 1 THEN RETURN 0 END;
  IF Kernel.BaseOf(gt) # wt THEN RETURN 0 END;
  IF Kernel.SizeOf(gt) <= Kernel.SizeOf(wt) THEN RETURN 0 END;  (* extra field *)
  RETURN 1
END GadgetReflection;

(** Time advances monotonically. Returns 1 if t2 >= t1, 0 otherwise. *)
PROCEDURE TimeMonotonic*(): INTEGER;
  VAR t1, t2: LONGINT;
BEGIN
  t1 := Kernel.Time();
  t2 := Kernel.Time();
  IF t2 < t1 THEN RETURN 0 END;
  IF t1 <= 0 THEN RETURN 0 END;
  RETURN 1
END TimeMonotonic;

(** NewObj typed allocation — call Kernel.NewObj directly and verify
    the returned pointer round-trips through TypeOf. *)
PROCEDURE NewObjRoundTrip*(): INTEGER;
  VAR p, w: Widget; tw, t: Kernel.Type;
BEGIN
  NEW(w);
  tw := Kernel.TypeOf(w);
  Kernel.NewObj(p, tw);
  IF p = NIL THEN RETURN 0 END;
  t := Kernel.TypeOf(p);
  IF t # tw THEN RETURN 0 END;
  RETURN 1
END NewObjRoundTrip;

(** Verify GetTypeName returns the bare type name. Uses `Kernel.Name`
    (the imported alias of `ARRAY 256 OF CHAR`) — this exercises the
    cross-module-array-alias resolution that was broken at one point
    and is now fixed in newcp-ir's map_semantic_type. *)
PROCEDURE WidgetTypeNameMatches*(): INTEGER;
  VAR w: Widget; t: Kernel.Type;
      name: Kernel.Name;
      i: INTEGER; expected: ARRAY 12 OF CHAR;
BEGIN
  NEW(w);
  t := Kernel.TypeOf(w);
  IF t = NIL THEN RETURN 0 END;
  Kernel.GetTypeName(t, name);
  expected := "WidgetDesc";
  i := 0;
  WHILE expected[i] # 0X DO
    IF name[i] # expected[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF name[i] # 0X THEN RETURN 0 END;       (* terminator *)
  RETURN 1
END WidgetTypeNameMatches;

(** Verify GetQualifiedTypeName returns the qualified Module.Type form. *)
PROCEDURE WidgetQualifiedTypeName*(): INTEGER;
  VAR w: Widget; t: Kernel.Type;
      name: Kernel.Name;
      i: INTEGER; expected: ARRAY 32 OF CHAR;
BEGIN
  NEW(w);
  t := Kernel.TypeOf(w);
  Kernel.GetQualifiedTypeName(t, name);
  expected := "KernelProbe.WidgetDesc";
  i := 0;
  WHILE expected[i] # 0X DO
    IF name[i] # expected[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF name[i] # 0X THEN RETURN 0 END;
  RETURN 1
END WidgetQualifiedTypeName;

(** Kernel.ThisMod resolves a registered native module to a non-NIL
    handle, and returns NIL for unknown names. *)
PROCEDURE ThisModResolvesKnownModule*(): INTEGER;
  VAR m: Kernel.Module;
BEGIN
  m := Kernel.ThisMod("Console");      (* registered at bootstrap *)
  IF m = NIL THEN RETURN 0 END;
  m := Kernel.ThisMod("Math");
  IF m = NIL THEN RETURN 0 END;
  m := Kernel.ThisMod("DefinitelyDoesNotExist");
  IF m # NIL THEN RETURN 0 END;
  RETURN 1
END ThisModResolvesKnownModule;

(** Kernel.ThisType returns NIL for an unknown (module, type) pair. *)
PROCEDURE ThisTypeNilWhenUnseen*(): INTEGER;
  VAR m: Kernel.Module; t: Kernel.Type;
BEGIN
  m := Kernel.ThisMod("Console");
  IF m = NIL THEN RETURN 0 END;
  t := Kernel.ThisType(m, "NoSuchType");
  IF t # NIL THEN RETURN 0 END;
  RETURN 1
END ThisTypeNilWhenUnseen;

(** Codegen emits per-module `__init_types` that registers every
    declared TypeDesc; the loader auto-registers each compiled CP
    module's name with `Kernel.ThisMod`. End-to-end:
    `Kernel.ThisMod("KernelProbe")` returns the module handle,
    `Kernel.ThisType(_, "WidgetDesc")` returns the TypeDesc, and
    that TypeDesc identity-equals `Kernel.TypeOf(w)` for a freshly
    NEW'd `Widget`. *)
PROCEDURE ThisTypeFindsRegisteredType*(): INTEGER;
  VAR w: Widget; td, looked_up: Kernel.Type; m: Kernel.Module;
BEGIN
  NEW(w);
  td := Kernel.TypeOf(w);
  IF td = NIL THEN RETURN 0 END;

  m := Kernel.ThisMod("KernelProbe");
  IF m = NIL THEN RETURN 0 END;
  looked_up := Kernel.ThisType(m, "WidgetDesc");
  IF looked_up = NIL THEN RETURN 0 END;
  IF looked_up # td THEN RETURN 0 END;

  (* Negative path: a name not declared by this module returns NIL
     even when the module handle is valid. *)
  IF Kernel.ThisType(m, "NoSuchTypeDesc") # NIL THEN RETURN 0 END;
  RETURN 1
END ThisTypeFindsRegisteredType;

PROCEDURE GetModNameMatchesRegisteredModule*(): INTEGER;
  VAR m: Kernel.Module; name: Kernel.Name;
      i: INTEGER; expected: ARRAY 16 OF CHAR;
BEGIN
  m := Kernel.ThisMod("Console");
  IF m = NIL THEN RETURN 0 END;
  Kernel.GetModName(m, name);
  expected := "Console";
  i := 0;
  WHILE expected[i] # 0X DO
    IF name[i] # expected[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF name[i] # 0X THEN RETURN 0 END;
  RETURN 1
END GetModNameMatchesRegisteredModule;

PROCEDURE ModOfMatchesTypeOwner*(): INTEGER;
  VAR w: Widget; t: Kernel.Type; m: Kernel.Module;
      name: Kernel.Name;
      i: INTEGER; expected: ARRAY 16 OF CHAR;
BEGIN
  NEW(w);
  t := Kernel.TypeOf(w);
  IF t = NIL THEN RETURN 0 END;
  m := Kernel.ModOf(t);
  IF m = NIL THEN RETURN 0 END;
  Kernel.GetModName(m, name);
  expected := "KernelProbe";
  i := 0;
  WHILE expected[i] # 0X DO
    IF name[i] # expected[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF name[i] # 0X THEN RETURN 0 END;
  RETURN 1
END ModOfMatchesTypeOwner;

PROCEDURE ThisModFailureSetsLoaderResult*(): INTEGER;
  VAR m: Kernel.Module; res: INTEGER;
      m1, m2, m3: Kernel.Name;
      i: INTEGER;
      expected1: ARRAY 24 OF CHAR;
      expected2: ARRAY 16 OF CHAR;
      expected3: ARRAY 24 OF CHAR;
BEGIN
  m := Kernel.ThisMod("DefinitelyDoesNotExist");
  IF m # NIL THEN RETURN 0 END;

  Kernel.GetLoaderResult(res, m1, m2, m3);
  IF res # Kernel.objNotFound THEN RETURN 0 END;

  expected1 := "DefinitelyDoesNotExist";
  i := 0;
  WHILE expected1[i] # 0X DO
    IF m1[i] # expected1[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF m1[i] # 0X THEN RETURN 0 END;

  expected2 := "this-mod";
  i := 0;
  WHILE expected2[i] # 0X DO
    IF m2[i] # expected2[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF m2[i] # 0X THEN RETURN 0 END;

  expected3 := "module not found";
  i := 0;
  WHILE expected3[i] # 0X DO
    IF m3[i] # expected3[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF m3[i] # 0X THEN RETURN 0 END;
  RETURN 1
END ThisModFailureSetsLoaderResult;

PROCEDURE ThisTypeFailureSetsLoaderResult*(): INTEGER;
  VAR m: Kernel.Module; t: Kernel.Type; res: INTEGER;
      m1, m2, m3: Kernel.Name;
      i: INTEGER;
      expected1: ARRAY 16 OF CHAR;
      expected2: ARRAY 16 OF CHAR;
      expected3: ARRAY 24 OF CHAR;
BEGIN
  m := Kernel.ThisMod("KernelProbe");
  IF m = NIL THEN RETURN 0 END;

  t := Kernel.ThisType(m, "NoSuchTypeDesc");
  IF t # NIL THEN RETURN 0 END;

  Kernel.GetLoaderResult(res, m1, m2, m3);
  IF res # Kernel.objNotFound THEN RETURN 0 END;

  expected1 := "KernelProbe";
  i := 0;
  WHILE expected1[i] # 0X DO
    IF m1[i] # expected1[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF m1[i] # 0X THEN RETURN 0 END;

  expected2 := "NoSuchTypeDesc";
  i := 0;
  WHILE expected2[i] # 0X DO
    IF m2[i] # expected2[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF m2[i] # 0X THEN RETURN 0 END;

  expected3 := "type not found";
  i := 0;
  WHILE expected3[i] # 0X DO
    IF m3[i] # expected3[i] THEN RETURN 0 END;
    INC(i)
  END;
  IF m3[i] # 0X THEN RETURN 0 END;
  RETURN 1
END ThisTypeFailureSetsLoaderResult;

PROCEDURE ReflectionSuccessClearsLoaderResult*(): INTEGER;
  VAR m: Kernel.Module; t: Kernel.Type; res: INTEGER;
      m1, m2, m3: Kernel.Name;
BEGIN
  IF Kernel.ThisMod("DefinitelyDoesNotExist") # NIL THEN RETURN 0 END;
  Kernel.GetLoaderResult(res, m1, m2, m3);
  IF res = Kernel.none THEN RETURN 0 END;

  m := Kernel.ThisMod("KernelProbe");
  IF m = NIL THEN RETURN 0 END;
  t := Kernel.ThisType(m, "WidgetDesc");
  IF t = NIL THEN RETURN 0 END;

  Kernel.GetLoaderResult(res, m1, m2, m3);
  IF res # Kernel.none THEN RETURN 0 END;
  IF (m1[0] # 0X) OR (m2[0] # 0X) OR (m3[0] # 0X) THEN RETURN 0 END;
  RETURN 1
END ReflectionSuccessClearsLoaderResult;

PROCEDURE GetModNameNilYieldsEmptyString*(): INTEGER;
  VAR name: Kernel.Name;
BEGIN
  Kernel.GetModName(NIL, name);
  IF name[0] # 0X THEN RETURN 0 END;
  RETURN 1
END GetModNameNilYieldsEmptyString;

PROCEDURE ModOfNilYieldsNil*(): INTEGER;
BEGIN
  IF Kernel.ModOf(NIL) # NIL THEN RETURN 0 END;
  RETURN 1
END ModOfNilYieldsNil;

PROCEDURE CleanLookupLeavesLoaderResultClear*(): INTEGER;
  VAR m: Kernel.Module; res: INTEGER;
      m1, m2, m3: Kernel.Name;
BEGIN
  m := Kernel.ThisMod("Console");
  IF m = NIL THEN RETURN 0 END;

  Kernel.GetLoaderResult(res, m1, m2, m3);
  IF res # Kernel.none THEN RETURN 0 END;
  IF (m1[0] # 0X) OR (m2[0] # 0X) OR (m3[0] # 0X) THEN RETURN 0 END;
  RETURN 1
END CleanLookupLeavesLoaderResultClear;

END KernelProbe.
