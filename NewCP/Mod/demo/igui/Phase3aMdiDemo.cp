MODULE Phase3aMdiDemo;

(* iGui Phase 3a acceptance demo. Opens two MDI children with
   different titles, prints child events to the console, and exits
   cleanly when the frame closes.

   Run with:
     newcp-driver run-igui Phase3aMdiDemo.Run

   Each child paints a per-child background color (deterministic by
   child id) so the two are visually distinct. Drag the children
   inside the MDI frame, resize them, close them with the system
   button — events should print on the console. *)

IMPORT iGui, Console;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
    childA, childB: INTEGER;
BEGIN
  Console.WriteShortString("Phase 3a MDI demo: opening two children..."); Console.WriteLn;

  ok := iGui.OpenChild("Document A", childA);
  IF ok = 0 THEN
    Console.WriteShortString("OpenChild A failed"); Console.WriteLn;
    RETURN
  END;
  Console.WriteShortString("opened child A id="); Console.WriteInt(childA); Console.WriteLn;

  ok := iGui.OpenChild("Document B", childB);
  IF ok = 0 THEN
    Console.WriteShortString("OpenChild B failed"); Console.WriteLn;
    RETURN
  END;
  Console.WriteShortString("opened child B id="); Console.WriteInt(childB); Console.WriteLn;

  Console.WriteShortString("ready; close the frame to exit"); Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvResize THEN
        Console.WriteShortString("[resize] child="); Console.WriteInt(childId);
        Console.WriteShortString(" "); Console.WriteInt(p1);
        Console.WriteShortString("x"); Console.WriteInt(p2); Console.WriteLn
      ELSIF kind = iGui.EvClose THEN
        Console.WriteShortString("[close] child="); Console.WriteInt(childId); Console.WriteLn
      ELSIF kind = iGui.EvFrameClose THEN
        Console.WriteShortString("[frame-close]"); Console.WriteLn;
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase3aMdiDemo.
