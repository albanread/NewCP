MODULE Phase3cGeometryDemo;

(* iGui Phase 3c acceptance demo. Two MDI children, each painting a
   distinct geometric scene that exercises the new ellipse / circle /
   arc primitives. Also reads the children's effective DPI on open
   and sets a per-child cursor.

   Run:
     newcp-driver run-igui Phase3cGeometryDemo.Run
*)

IMPORT iGui, Console;

CONST
  Pi = 3.141592653589793;

PROCEDURE PaintChildA(childId: INTEGER);
  VAR
    cx, cy, r, rot, halfAp, ht: REAL;
    i: INTEGER;
    ok: INTSHORT;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.14, 0.22, 1.0);                  (* deep slate blue *)

  (* A row of solid circles with shifting hue *)
  cx := 80.0; cy := 100.0; r := 30.0;
  i := 0;
  WHILE i < 5 DO
    iGui.EmitFillCircle(cx, cy, r,
                        0.85 - 0.08 * (i + 1),
                        0.40 + 0.05 * (i + 1),
                        0.30 + 0.05 * (i + 1),
                        1.0);
    cx := cx + 80.0;
    INC(i)
  END;

  (* An outline ellipse below the circles *)
  iGui.EmitStrokeOval(40.0, 180.0, 460.0, 280.0, 1.5,
                      0.95, 0.95, 0.95, 1.0);

  (* A filled ellipse marker *)
  iGui.EmitFillOval(500.0, 180.0, 600.0, 280.0,
                    0.55, 0.85, 1.00, 1.0);

  (* A pacman: a circle with a missing wedge, shown as a stroked arc.
     rotation_rad = 0 means the arc opens to the right;
     half_aperture covers everything except a 60° wedge → 5π/6. *)
  cx := 200.0; cy := 400.0; r := 60.0;
  rot := 0.0;
  halfAp := 5.0 * Pi / 6.0;
  ht := 3.0;
  iGui.EmitDrawArc(cx, cy, r, rot, halfAp, ht,
                   1.00, 0.85, 0.30, 1.0);

  (* A stroked circle for contrast *)
  iGui.EmitStrokeCircle(420.0, 400.0, 50.0, 2.0,
                        0.50, 0.95, 0.60, 1.0);

  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[A] SubmitBatch failed"); Console.WriteLn
  END
END PaintChildA;

PROCEDURE PaintChildB(childId: INTEGER);
  VAR
    cx, cy, r: REAL;
    ok: INTSHORT;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.12, 0.18, 0.14, 1.0);                  (* deep forest *)

  (* Concentric stroked circles *)
  cx := 250.0; cy := 200.0;
  r := 30.0;
  WHILE r <= 130.0 DO
    iGui.EmitStrokeCircle(cx, cy, r, 1.2,
                          0.85, 0.85, 0.45, 1.0);
    r := r + 20.0
  END;

  (* A solid circle in the middle *)
  iGui.EmitFillCircle(cx, cy, 14.0,
                      1.0, 0.55, 0.20, 1.0);

  (* A semicircle arc opening upward (rotation 3π/2 = pointing up,
     half-aperture π/2 covers π radians → top half) *)
  iGui.EmitDrawArc(cx, 380.0, 80.0, 3.0 * Pi / 2.0, Pi / 2.0, 2.5,
                   0.55, 0.85, 1.00, 1.0);

  (* A small filled ellipse marker *)
  iGui.EmitFillOval(500.0, 60.0, 600.0, 100.0,
                    1.0, 0.65, 0.30, 1.0);

  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[B] SubmitBatch failed"); Console.WriteLn
  END
END PaintChildB;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
    childA, childB: INTEGER;
    dpiX, dpiY: REAL;
BEGIN
  Console.WriteShortString("Phase 3c demo: ovals, circles, arcs, cursor, DPI");
  Console.WriteLn;

  ok := iGui.OpenChild("Document A — circles + arcs", childA);
  IF ok = 0 THEN RETURN END;
  ok := iGui.OpenChild("Document B — concentric + arc", childB);
  IF ok = 0 THEN RETURN END;

  (* Read each child's effective DPI *)
  IF iGui.GetDpi(childA, dpiX, dpiY) # 0 THEN
    Console.WriteShortString("[A] dpi="); Console.WriteReal(dpiX);
    Console.WriteShortString(" x "); Console.WriteReal(dpiY); Console.WriteLn
  END;
  IF iGui.GetDpi(childB, dpiX, dpiY) # 0 THEN
    Console.WriteShortString("[B] dpi="); Console.WriteReal(dpiX);
    Console.WriteShortString(" x "); Console.WriteReal(dpiY); Console.WriteLn
  END;

  (* Set distinct cursors so we can visually verify SetCursor works *)
  iGui.SetCursor(childA, iGui.CrCrosshair);
  iGui.SetCursor(childB, iGui.CrHand);

  PaintChildA(childA);
  PaintChildB(childB);

  Console.WriteShortString("close the frame to exit"); Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvResize THEN
        IF childId = childA THEN PaintChildA(childA)
        ELSIF childId = childB THEN PaintChildB(childB)
        END
      ELSIF kind = iGui.EvDpiChange THEN
        Console.WriteShortString("[dpi-change] child="); Console.WriteInt(childId);
        Console.WriteShortString(" dpi="); Console.WriteInt(p1 DIV 100);
        Console.WriteShortString(" x "); Console.WriteInt(p2 DIV 100); Console.WriteLn
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase3cGeometryDemo.
