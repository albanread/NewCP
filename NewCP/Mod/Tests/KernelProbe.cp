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

END KernelProbe.
