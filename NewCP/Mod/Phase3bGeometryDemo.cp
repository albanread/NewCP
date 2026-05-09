MODULE Phase3bGeometryDemo;

(* iGui Phase 3b acceptance demo. Opens two MDI children, paints a
   distinct geometric scene into each via the surface batch API, and
   exits when the frame closes.

   Run with:
     newcp-driver run-igui Phase3bGeometryDemo.Run

   Child A: dark blue background, a row of warm-colored filled
            rectangles, an outline rectangle around the group, and
            a diagonal line.
   Child B: dark green background, a stack of stroked rounded
            rectangles, with a horizontal divider line. *)

IMPORT iGui, Console;

PROCEDURE PaintChildA(childId: INTEGER);
  VAR i: INTEGER;
      ok: INTSHORT;
      x, hueR, hueG, hueB: REAL;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.14, 0.22, 1.0);                 (* deep slate blue *)

  (* Row of five filled rounded rectangles with shifting hue *)
  x := 60.0;
  hueR := 0.85;
  hueG := 0.40;
  hueB := 0.20;
  i := 0;
  WHILE i < 5 DO
    iGui.EmitFillRect(x, 80.0, x + 90.0, 200.0, 12.0,
                      hueR, hueG, hueB, 1.0);
    x    := x    + 110.0;
    hueR := hueR - 0.10;
    hueG := hueG + 0.05;
    hueB := hueB + 0.05;
    INC(i)
  END;

  (* Outline rectangle around the row *)
  iGui.EmitStrokeRect(50.0, 70.0, 620.0, 210.0,
                      8.0, 1.5,
                      0.95, 0.95, 0.95, 1.0);

  (* Diagonal line across *)
  iGui.EmitDrawLine(20.0, 280.0, 700.0, 360.0, 1.5,
                    0.55, 0.85, 1.00, 1.0);

  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[A] SubmitBatch failed"); Console.WriteLn
  END
END PaintChildA;

PROCEDURE PaintChildB(childId: INTEGER);
  VAR i: INTEGER;
      ok: INTSHORT;
      y, t: REAL;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.20, 0.14, 1.0);                 (* deep forest *)

  (* Stack of stroked rounded rectangles, increasing thickness *)
  y := 40.0;
  t := 1.0;
  i := 0;
  WHILE i < 4 DO
    iGui.EmitStrokeRect(50.0, y, 600.0, y + 50.0,
                        18.0, t,
                        0.90, 0.90, 0.50, 1.0);
    y := y + 70.0;
    t := t + 1.0;
    INC(i)
  END;

  (* Horizontal divider *)
  iGui.EmitDrawLine(20.0, 340.0, 700.0, 340.0, 0.75,
                    0.50, 0.95, 0.50, 1.0);

  (* A small filled square as a marker *)
  iGui.EmitFillRect(630.0, 50.0, 670.0, 90.0, 6.0,
                    1.0, 0.55, 0.20, 1.0);

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
BEGIN
  Console.WriteShortString("Phase 3b geometry demo: opening children...");
  Console.WriteLn;

  ok := iGui.OpenChild("Document A — fills and outline", childA);
  IF ok = 0 THEN RETURN END;
  ok := iGui.OpenChild("Document B — stacked outlines", childB);
  IF ok = 0 THEN RETURN END;

  PaintChildA(childA);
  PaintChildB(childB);

  Console.WriteShortString("submitted initial scenes; close the frame to exit");
  Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvResize THEN
        IF childId = childA THEN PaintChildA(childA)
        ELSIF childId = childB THEN PaintChildB(childB)
        END
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase3bGeometryDemo.
