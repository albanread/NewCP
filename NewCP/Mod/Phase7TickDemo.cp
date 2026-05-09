MODULE Phase7TickDemo;

(* iGui animation-tick demo. Two MDI children:

   Document A (driven): subscribes to a 33ms tick (~30 fps). On each
     EvTick the demo advances a position counter and re-paints with
     a circle that bounces left/right inside the child's client area.

   Document B (static): no tick, painted once with a label.

   Run:
     newcp-driver run-igui Phase7TickDemo.Run
*)

IMPORT iGui, Console;

VAR
  posX: REAL;
  velX: REAL;

PROCEDURE PaintBouncer(childId: INTEGER);
  VAR
    ok: INTSHORT;
    cx, cy, r: REAL;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.13, 0.18, 1.0);

  cy := 180.0;
  r := 32.0;
  cx := posX;
  iGui.EmitFillCircle(cx, cy, r,
                      1.00, 0.55, 0.30, 1.0);

  (* Trail of fading circles *)
  iGui.EmitFillCircle(cx - velX, cy, r * 0.85,
                      1.00, 0.55, 0.30, 0.4);
  iGui.EmitFillCircle(cx - velX * 2.0, cy, r * 0.65,
                      1.00, 0.55, 0.30, 0.2);

  (* Floor line *)
  iGui.EmitDrawLine(20.0, 230.0, 760.0, 230.0, 0.5,
                    0.55, 0.85, 1.00, 1.0);
  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[A] SubmitBatch failed"); Console.WriteLn
  END
END PaintBouncer;

PROCEDURE PaintStatic(childId: INTEGER);
  VAR ok: INTSHORT;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.18, 0.13, 1.0);
  iGui.EmitDrawTextRun(
    "static document — no ticks here",
    40.0, 100.0, 22.0,
    "Segoe UI",
    400, iGui.FsNormal, iGui.FwNormal,
    "en-us", -1.0, iGui.AlignLeading, iGui.TrimNone,
    0.85, 0.95, 0.85, 1.0);
  iGui.EmitDrawTextRun(
    "the bouncer next door is repainting at 30fps",
    40.0, 140.0, 14.0,
    "Segoe UI",
    400, iGui.FsItalic, iGui.FwNormal,
    "en-us", -1.0, iGui.AlignLeading, iGui.TrimNone,
    0.55, 0.75, 0.55, 1.0);
  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[B] SubmitBatch failed"); Console.WriteLn
  END
END PaintStatic;

PROCEDURE Step;
BEGIN
  posX := posX + velX;
  IF posX > 700.0 THEN
    posX := 700.0;
    velX := -velX
  ELSIF posX < 80.0 THEN
    posX := 80.0;
    velX := -velX
  END
END Step;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
    bouncer, statik: INTEGER;
    tickCount: INTEGER;
BEGIN
  Console.WriteShortString("Tick demo: 30fps animation via Win32 timer + EvTick");
  Console.WriteLn;

  posX := 100.0;
  velX := 8.0;
  tickCount := 0;

  ok := iGui.OpenChild("Bouncer (animated)", bouncer);
  IF ok = 0 THEN RETURN END;
  ok := iGui.OpenChild("Static (no tick)", statik);
  IF ok = 0 THEN RETURN END;

  PaintStatic(statik);
  PaintBouncer(bouncer);

  ok := iGui.SetRedrawRate(bouncer, 33);
  IF ok = 0 THEN
    Console.WriteShortString("SetRedrawRate failed"); Console.WriteLn;
    RETURN
  END;

  Console.WriteShortString("ticking; close the frame to exit"); Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvTick THEN
        IF childId = bouncer THEN
          Step;
          PaintBouncer(bouncer);
          INC(tickCount);
          IF tickCount MOD 30 = 0 THEN
            Console.WriteShortString("[tick] count="); Console.WriteInt(tickCount);
            Console.WriteShortString(" posX="); Console.WriteReal(posX);
            Console.WriteLn
          END
        END
      ELSIF kind = iGui.EvResize THEN
        IF childId = bouncer THEN PaintBouncer(bouncer)
        ELSIF childId = statik THEN PaintStatic(statik)
        END
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase7TickDemo.
