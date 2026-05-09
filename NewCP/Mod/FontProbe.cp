MODULE FontProbe;

(* End-to-end smoke test for the Fonts / HostFontsSys / HostFonts /
   iGui.font_metrics chain.

   Asks the host font directory for the default font, measures its
   cell, then measures a known string. Prints the values to the
   console. Run with:

     newcp-driver run-igui FontProbe.Run

   Expected output: ascent / descent / line-height / advance values
   for Cascadia Mono at 10pt (~125000 sub-mm = ~13 DIPs body), and a
   plausible width for a sample string. *)

IMPORT iGui, Fonts, HostFonts, Console;

PROCEDURE Run*;
  VAR
    f:         Fonts.Font;
    asc, dsc, w, sw: INTEGER;
    sample:    ARRAY 64 OF SHORTCHAR;
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok:        INTSHORT;
BEGIN
  Console.WriteShortString("FontProbe: measuring default font"); Console.WriteLn;

  f := Fonts.dir.Default();
  IF f = NIL THEN
    Console.WriteShortString("FontProbe: Fonts.dir.Default returned NIL");
    Console.WriteLn;
    RETURN
  END;

  f.GetBounds(asc, dsc, w);
  Console.WriteShortString("default font cell (BB sub-mm units):"); Console.WriteLn;
  Console.WriteShortString("  ascent  = "); Console.WriteInt(asc); Console.WriteLn;
  Console.WriteShortString("  descent = "); Console.WriteInt(dsc); Console.WriteLn;
  Console.WriteShortString("  width   = "); Console.WriteInt(w); Console.WriteLn;

  sample := "the quick brown fox";
  sw := f.SStringWidth(sample);
  Console.WriteShortString("string width (BB sub-mm) for '"); Console.WriteShortString(sample);
  Console.WriteShortString("' = "); Console.WriteInt(sw); Console.WriteLn;

  (* Open a child so the iGui frame stays up; close it from the user
     side to end the demo. *)
  ok := iGui.OpenChild("FontProbe", childId);
  IF ok = 0 THEN RETURN END;
  Console.WriteShortString("close the frame to exit"); Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF (ok # 0) & (kind = iGui.EvFrameClose) THEN EXIT END
  UNTIL FALSE
END Run;

END FontProbe.
