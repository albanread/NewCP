MODULE MechanismsExtBase;
(*
   Workout for the `Mechanisms` slice.

   `Mechanisms` is a thin trampoline: every public proc
   forwards to the installed `Hook` via virtual dispatch.
   This test:

   1. Defines `MyHookDesc` extending `Mechanisms.HookDesc` and
      overrides every ABSTRACT method.  Only the three
      methods we drive in `Run` record inputs in module
      vars; the others are stub overrides so `NEW(MyHook)`
      is well-defined.
   2. Installs the hook via `Mechanisms.SetHook`.
   3. Calls three trampolines: `MarkFocusBorder`,
      `FocusBorderCursor`, `SelBorderCursor`.
   4. Verifies the overrides fired and the arguments round-
      tripped through the cross-module indirection.

   The path under test:

       Run (this module)
            ↓
       Mechanisms.<Trampoline>(...)
            ↓ (hook.<Method>)
       MyHookDesc.<Method>     ← cross-module virtual dispatch
            ↓
       module-level vars stamped here

   Returns a packed int proving each trampoline reached its
   override.
*)

    IMPORT Mechanisms, Views;

    TYPE
        MyHookDesc* = RECORD (Mechanisms.HookDesc) END;
        MyHook* = POINTER TO MyHookDesc;


    VAR
        markFocusFired*: BOOLEAN;
        focusL*, focusT*, focusR*, focusB*: INTEGER;
        focusCursorArgL*, focusCursorArgT*: INTEGER;
        selCursorArgL*, selCursorArgT*: INTEGER;


    (* Method we exercise — records the rectangle and show flag. *)
    PROCEDURE (hk: MyHookDesc) MarkFocusBorder*
        (host: Views.Frame; focus: Views.View;
         l, t, r, b: INTEGER; show: BOOLEAN);
    BEGIN
        markFocusFired := show;
        focusL := l;  focusT := t;  focusR := r;  focusB := b
    END MarkFocusBorder;

    (* Stub override — required by ABSTRACT chain. *)
    PROCEDURE (hk: MyHookDesc) MarkSingletonBorder*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER; show: BOOLEAN);
    BEGIN
    END MarkSingletonBorder;

    (* Method we exercise — sentinel cursor return so test
       packs distinct values. *)
    PROCEDURE (hk: MyHookDesc) FocusBorderCursor*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER;
         x, y: INTEGER): INTEGER;
    BEGIN
        focusCursorArgL := l;  focusCursorArgT := t;
        RETURN Mechanisms.inside       (* -1 *)
    END FocusBorderCursor;

    (* Method we exercise — different sentinel. *)
    PROCEDURE (hk: MyHookDesc) SelBorderCursor*
        (host: Views.Frame; view: Views.View;
         l, t, r, b: INTEGER;
         x, y: INTEGER): INTEGER;
    BEGIN
        selCursorArgL := l;  selCursorArgT := t;
        RETURN Mechanisms.outside      (* -2 *)
    END SelBorderCursor;

    (* Stub overrides for the remaining ABSTRACTs.  Each just
       satisfies the override-completeness check. *)
    PROCEDURE (hk: MyHookDesc) TrackToResize*
        (host: Views.Frame; view: Views.View;
         minW, maxW, minH, maxH: INTEGER;
         VAR l, t, r, b: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
    END TrackToResize;

    PROCEDURE (hk: MyHookDesc) TrackToDrop*
        (source: Views.Frame; view: Views.View;
         isSingle: BOOLEAN; w, h, rx, ry: INTEGER;
         VAR dest: Views.Frame; VAR destX, destY: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
    END TrackToDrop;

    PROCEDURE (hk: MyHookDesc) TrackToPick*
        (source: Views.Frame;
         VAR dest: Views.Frame; VAR destX, destY: INTEGER;
         VAR op: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
    END TrackToPick;

    PROCEDURE (hk: MyHookDesc) PopUpAndSelect*
        (f: Views.Frame;
         n, this: INTEGER;
         s: ARRAY OF ARRAY OF CHAR;
         enabled, checked: ARRAY OF BOOLEAN;
         VAR i: INTEGER;
         VAR x, y: INTEGER;
         VAR buttons: SET);
    BEGIN
    END PopUpAndSelect;


    (* -- Driver --------------------------------------------------------- *)

    PROCEDURE Run* (): INTEGER;
        VAR me: MyHook; cur1, cur2: INTEGER; packed: INTEGER;
    BEGIN
        markFocusFired := FALSE;
        focusL := 0; focusT := 0; focusR := 0; focusB := 0;
        focusCursorArgL := 0;  focusCursorArgT := 0;
        selCursorArgL := 0;    selCursorArgT := 0;

        NEW(me);
        Mechanisms.SetHook(me);

        IF ~Mechanisms.HookIsInstalled() THEN RETURN -1 END;

        (* Stage 1: MarkFocusBorder trampoline. *)
        Mechanisms.MarkFocusBorder(NIL, NIL, 11, 22, 33, 44, TRUE);
        IF ~markFocusFired THEN RETURN -2 END;
        IF focusL # 11 THEN RETURN -3 END;
        IF focusT # 22 THEN RETURN -4 END;
        IF focusR # 33 THEN RETURN -5 END;
        IF focusB # 44 THEN RETURN -6 END;

        (* Stage 2: FocusBorderCursor trampoline returns inside. *)
        cur1 := Mechanisms.FocusBorderCursor(NIL, NIL, 5, 6, 7, 8, 0, 0);
        IF cur1 # Mechanisms.inside THEN RETURN -10 END;
        IF focusCursorArgL # 5 THEN RETURN -11 END;
        IF focusCursorArgT # 6 THEN RETURN -12 END;

        (* Stage 3: SelBorderCursor trampoline returns outside. *)
        cur2 := Mechanisms.SelBorderCursor(NIL, NIL, 1, 2, 3, 4, 0, 0);
        IF cur2 # Mechanisms.outside THEN RETURN -20 END;
        IF selCursorArgL # 1 THEN RETURN -21 END;
        IF selCursorArgT # 2 THEN RETURN -22 END;

        (* Pack a value that proves all three trampolines ran:
             focusL(11)*10000 + focusT(22)*100 + focusR(33)
             = 110000 + 2200 + 33 = 112233.
           Add 1 if MarkFocusBorder.show=TRUE was honoured.
           = 112234 *)
        packed := focusL * 10000 + focusT * 100 + focusR;
        IF markFocusFired THEN packed := packed + 1 END;
        RETURN packed       (* expected 112234 *)
    END Run;

END MechanismsExtBase.
