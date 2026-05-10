MODULE Phase5CompositionDemo;

(* iGui Phase 5 acceptance demo: composition, overlays, paths,
   system colors. Two MDI children:

   Document A:
     - A clipped scene: PushClipRect, draw a row of stripes that
       extends beyond the clip rect, PopClipRect.
     - A coordinate-translated scene with PushOffset, draw a small
       sub-scene, PopOffset.
     - A focus ring outlining the lot.
     - Caret and SelectionRange overlays.

   Document B:
     - A vector star drawn with the path builder (MoveTo + 9 LineTos
       + Close, filled and stroked).
     - A rounded badge built with QuadTo curves, filled with the
       system selection color.
     - Three MarkRects in different modes (Highlight, Dim25, Dim50)
       over a colored background, demonstrating they read the system
       palette.

   Run:
     newcp-driver run-igui Phase5CompositionDemo.Run
*)

IMPORT iGui, Console;

CONST
  Pi = 3.141592653589793;

PROCEDURE PaintChildA(childId: INTEGER);
  VAR
    ok: INTSHORT;
    selR, selG, selB, selA: REAL;
    i: INTEGER;
    x: REAL;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.12, 0.18, 1.0);

  (* Clipped row of stripes — the stripes extend beyond the clip rect
     vertically; only the middle band shows through. *)
  iGui.EmitPushClipRect(40.0, 80.0, 600.0, 160.0);
  x := 50.0;
  i := 0;
  WHILE i < 12 DO
    iGui.EmitFillRect(x, 40.0, x + 30.0, 220.0, 0.0,
                      0.30 + 0.05 * i, 0.55, 0.95 - 0.04 * i, 1.0);
    x := x + 45.0;
    INC(i)
  END;
  iGui.EmitPopClipRect;

  (* Translated sub-scene: same row of three squares drawn at (0,0)
     in local coords, but rendered offset by (50, 250). *)
  iGui.EmitPushOffset(50.0, 250.0);
  iGui.EmitFillRect(0.0, 0.0, 40.0, 40.0, 4.0, 1.0, 0.45, 0.30, 1.0);
  iGui.EmitFillRect(60.0, 0.0, 100.0, 40.0, 4.0, 1.0, 0.65, 0.30, 1.0);
  iGui.EmitFillRect(120.0, 0.0, 160.0, 40.0, 4.0, 1.0, 0.85, 0.30, 1.0);
  iGui.EmitPopOffset;

  (* Caret as a 2-DIP cyan vertical bar *)
  iGui.EmitCaret(280.0, 250.0, 282.0, 290.0,
                 0.55, 1.00, 1.00, 1.0);

  (* Selection range using the system selection color, semi-transparent *)
  IF iGui.SystemColor(iGui.ScSelectionBg, selR, selG, selB, selA) # 0 THEN
    iGui.EmitSelectionRange(310.0, 250.0, 460.0, 290.0,
                            selR, selG, selB, 0.40)
  END;

  (* Focus ring around the whole composition *)
  iGui.EmitFocusRing(30.0, 70.0, 610.0, 310.0, 6.0, 1.5,
                     0.55, 0.85, 1.00, 1.0);

  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[A] SubmitBatch failed"); Console.WriteLn
  END
END PaintChildA;

(* Tiny taylor-series cos / sin to keep the demo zero-import. NewCP
   has Math.Cos / Math.Sin but we'd rather not depend on it from the
   iGui demo. 6-term series — accurate to ~1e-5 over [-π, π]. *)
PROCEDURE MathCos(x: REAL): REAL;
  VAR
    x2, term, sum, denomA, denomB: REAL;
    i: INTEGER;
BEGIN
  WHILE x > Pi DO x := x - 2.0 * Pi END;
  WHILE x < -Pi DO x := x + 2.0 * Pi END;
  x2 := x * x;
  term := 1.0;
  sum := 1.0;
  denomA := 1.0;
  denomB := 2.0;
  i := 1;
  WHILE i <= 6 DO
    term := -term * x2 / (denomA * denomB);
    sum := sum + term;
    denomA := denomA + 2.0;
    denomB := denomB + 2.0;
    INC(i)
  END;
  RETURN sum
END MathCos;

PROCEDURE MathSin(x: REAL): REAL;
BEGIN
  RETURN MathCos(Pi / 2.0 - x)
END MathSin;

(* Build a 5-pointed star by hand, centered at (cx, cy), with outer
   radius rOuter and inner radius rInner. *)
PROCEDURE BuildStar(cx, cy, rOuter, rInner: REAL);
  VAR
    i: INTEGER;
    angle, x, y, step: REAL;
    radius: REAL;
BEGIN
  step := Pi / 5.0;
  angle := -Pi / 2.0;
  i := 0;
  WHILE i < 10 DO
    IF (i MOD 2) = 0 THEN radius := rOuter
    ELSE radius := rInner
    END;
    x := cx + radius * MathCos(angle);
    y := cy + radius * MathSin(angle);
    IF i = 0 THEN iGui.PathMoveTo(x, y)
    ELSE iGui.PathLineTo(x, y)
    END;
    angle := angle + step;
    INC(i)
  END;
  iGui.PathClose
END BuildStar;

PROCEDURE PaintChildB(childId: INTEGER);
  VAR
    ok: INTSHORT;
    selR, selG, selB, selA: REAL;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.14, 0.10, 0.18, 1.0);

  (* Filled + stroked star *)
  iGui.PathBegin;
  BuildStar(150.0, 150.0, 70.0, 28.0);
  ok := iGui.EmitPath(
    1, 1.00, 0.85, 0.40, 1.0,            (* fill *)
    1, 1.5, iGui.CapRound, iGui.JoinRound, 4.0, 0,
    0.50, 0.30, 0.10, 1.0);              (* stroke *)
  IF ok = 0 THEN
    Console.WriteShortString("[B] star EmitPath failed"); Console.WriteLn
  END;

  (* Rounded badge using two QuadTo curves *)
  iGui.PathBegin;
  iGui.PathMoveTo(280.0, 100.0);
  iGui.PathLineTo(420.0, 100.0);
  iGui.PathQuadTo(460.0, 100.0, 460.0, 140.0);
  iGui.PathLineTo(460.0, 200.0);
  iGui.PathQuadTo(460.0, 240.0, 420.0, 240.0);
  iGui.PathLineTo(280.0, 240.0);
  iGui.PathQuadTo(240.0, 240.0, 240.0, 200.0);
  iGui.PathLineTo(240.0, 140.0);
  iGui.PathQuadTo(240.0, 100.0, 280.0, 100.0);
  iGui.PathClose;
  IF iGui.SystemColor(iGui.ScSelectionBg, selR, selG, selB, selA) # 0 THEN
    ok := iGui.EmitPath(
      1, selR, selG, selB, 1.0,
      1, 1.0, iGui.CapFlat, iGui.JoinMiter, 4.0, 0,
      1.0, 1.0, 1.0, 0.6)
  END;

  (* Three mark-rect modes over a single colored background *)
  iGui.EmitFillRect(40.0, 290.0, 540.0, 360.0, 0.0,
                    0.55, 0.30, 0.85, 1.0);
  iGui.EmitMarkRect(40.0,  290.0, 200.0, 360.0, iGui.MarkHighlight);
  iGui.EmitMarkRect(200.0, 290.0, 360.0, 360.0, iGui.MarkDim25);
  iGui.EmitMarkRect(360.0, 290.0, 540.0, 360.0, iGui.MarkDim50);

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
  Console.WriteShortString("Phase 5 demo: composition + overlays + paths");
  Console.WriteLn;

  ok := iGui.OpenChild("Document A — clip + offset + overlays", childA);
  IF ok = 0 THEN RETURN END;
  ok := iGui.OpenChild("Document B — paths + marks", childB);
  IF ok = 0 THEN RETURN END;

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
      ELSIF kind = iGui.EvThemeChange THEN
        Console.WriteShortString("[theme-change] palette refreshed"); Console.WriteLn;
        PaintChildA(childA);
        PaintChildB(childB)
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase5CompositionDemo.
