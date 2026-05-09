MODULE Phase4TextDemo;

(* iGui Phase 4 acceptance demo: DirectWrite text + the three
   synchronous queries. Two MDI children:

   Document A:
     - A bold heading and a body paragraph in different fonts/sizes.
     - A measure-driven underline that exactly matches the heading's
       rendered width (round-trips through MeasureTextRun).

   Document B:
     - A monospaced text run.
     - A caret rendered at the position of character index 7
       (round-trips through PointAtCharIndex).

   Run:
     newcp-driver run-igui Phase4TextDemo.Run
*)

IMPORT iGui, Console;

CONST
  HeadingFamily = "Segoe UI";
  BodyFamily    = "Segoe UI";
  MonoFamily    = "Cascadia Mono";
  Locale        = "en-us";

PROCEDURE PaintChildA(childId: INTEGER);
  VAR
    ok: INTSHORT;
    w, h, ascent: REAL;
    lineCount: INTEGER;
    headingX, headingY: REAL;
BEGIN
  iGui.BeginBatch(childId);
  iGui.EmitClear(0.10, 0.12, 0.16, 1.0);

  headingX := 40.0;
  headingY := 50.0;

  iGui.EmitDrawTextRun(
    "iGui Phase 4 — DirectWrite text",
    headingX, headingY, 28.0,
    HeadingFamily,
    700, iGui.FsNormal, iGui.FwNormal,
    Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
    0.95, 0.95, 0.95, 1.0);

  iGui.EmitDrawTextRun(
    "Bold above. Below: a body paragraph with a longer string of words to demonstrate proportional advances and line metrics.",
    headingX, headingY + 50.0, 14.0,
    BodyFamily,
    400, iGui.FsNormal, iGui.FwNormal,
    Locale, 600.0, iGui.AlignLeading, iGui.TrimNone,
    0.85, 0.88, 0.92, 1.0);

  iGui.EmitDrawTextRun(
    "italic line in a different style",
    headingX, headingY + 140.0, 16.0,
    BodyFamily,
    400, iGui.FsItalic, iGui.FwNormal,
    Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
    0.85, 0.65, 1.00, 1.0);

  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[A] SubmitBatch failed"); Console.WriteLn
  END;

  (* Round-trip the heading through MeasureTextRun, then draw an
     underline of the exact rendered width. *)
  IF iGui.MeasureTextRun(
       childId,
       "iGui Phase 4 — DirectWrite text",
       28.0, HeadingFamily,
       700, iGui.FsNormal, iGui.FwNormal,
       Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
       w, h, ascent, lineCount) # 0 THEN
    Console.WriteShortString("[A] heading width="); Console.WriteReal(w);
    Console.WriteShortString(" height="); Console.WriteReal(h);
    Console.WriteShortString(" ascent="); Console.WriteReal(ascent);
    Console.WriteShortString(" lines="); Console.WriteInt(lineCount); Console.WriteLn;

    iGui.BeginBatch(childId);
    iGui.EmitClear(0.10, 0.12, 0.16, 1.0);
    iGui.EmitDrawTextRun(
      "iGui Phase 4 — DirectWrite text",
      headingX, headingY, 28.0,
      HeadingFamily,
      700, iGui.FsNormal, iGui.FwNormal,
      Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
      0.95, 0.95, 0.95, 1.0);
    iGui.EmitDrawTextRun(
      "Bold above. Below: a body paragraph with a longer string of words to demonstrate proportional advances and line metrics.",
      headingX, headingY + 50.0, 14.0,
      BodyFamily,
      400, iGui.FsNormal, iGui.FwNormal,
      Locale, 600.0, iGui.AlignLeading, iGui.TrimNone,
      0.85, 0.88, 0.92, 1.0);
    iGui.EmitDrawTextRun(
      "italic line in a different style",
      headingX, headingY + 140.0, 16.0,
      BodyFamily,
      400, iGui.FsItalic, iGui.FwNormal,
      Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
      0.85, 0.65, 1.00, 1.0);
    (* Underline matching exact heading width *)
    iGui.EmitDrawLine(
      headingX, headingY + h + 4.0,
      headingX + w, headingY + h + 4.0,
      1.0, 0.55, 0.85, 1.00, 1.0);
    ok := iGui.SubmitBatch();
    IF ok = 0 THEN
      Console.WriteShortString("[A] resubmit failed"); Console.WriteLn
    END
  ELSE
    Console.WriteShortString("[A] MeasureTextRun failed"); Console.WriteLn
  END
END PaintChildA;

PROCEDURE PaintChildB(childId: INTEGER);
  VAR
    ok: INTSHORT;
    text: ARRAY 64 OF SHORTCHAR;
    caretX, caretY, caretH, baseX, baseY: REAL;
BEGIN
  text := "monospaced sample text";
  baseX := 40.0; baseY := 80.0;

  iGui.BeginBatch(childId);
  iGui.EmitClear(0.12, 0.16, 0.12, 1.0);
  iGui.EmitDrawTextRun(
    text,
    baseX, baseY, 22.0,
    MonoFamily,
    400, iGui.FsNormal, iGui.FwNormal,
    Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
    0.85, 1.00, 0.85, 1.0);
  ok := iGui.SubmitBatch();
  IF ok = 0 THEN
    Console.WriteShortString("[B] SubmitBatch failed"); Console.WriteLn
  END;

  (* Position the caret at character index 7 (just before "ed sample"
     after "monospac"). *)
  IF iGui.PointAtCharIndex(
       childId, text, 22.0, MonoFamily,
       400, iGui.FsNormal, 7,
       caretX, caretY, caretH) # 0 THEN
    Console.WriteShortString("[B] char 7 -> ("); Console.WriteReal(caretX);
    Console.WriteShortString(", "); Console.WriteReal(caretY);
    Console.WriteShortString(") h="); Console.WriteReal(caretH); Console.WriteLn;

    iGui.BeginBatch(childId);
    iGui.EmitClear(0.12, 0.16, 0.12, 1.0);
    iGui.EmitDrawTextRun(
      text,
      baseX, baseY, 22.0,
      MonoFamily,
      400, iGui.FsNormal, iGui.FwNormal,
      Locale, -1.0, iGui.AlignLeading, iGui.TrimNone,
      0.85, 1.00, 0.85, 1.0);
    (* Caret as a 2-DIP-wide vertical line at the returned position *)
    iGui.EmitFillRect(
      baseX + caretX - 1.0, baseY + caretY,
      baseX + caretX + 1.0, baseY + caretY + caretH,
      0.0, 1.00, 1.00, 0.50, 1.0);
    ok := iGui.SubmitBatch();
    IF ok = 0 THEN
      Console.WriteShortString("[B] resubmit failed"); Console.WriteLn
    END
  ELSE
    Console.WriteShortString("[B] PointAtCharIndex failed"); Console.WriteLn
  END
END PaintChildB;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
    childA, childB: INTEGER;
BEGIN
  Console.WriteShortString("Phase 4 demo: DirectWrite text + sync queries");
  Console.WriteLn;

  ok := iGui.OpenChild("Document A — text + measure", childA);
  IF ok = 0 THEN RETURN END;
  ok := iGui.OpenChild("Document B — caret via hit-test", childB);
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
      ELSIF kind = iGui.EvFrameClose THEN
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase4TextDemo.
