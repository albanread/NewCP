MODULE HelloPixels;
(* The simplest possible "show pixels" demo.  Opens an iGui child
   window, paints a white background, a black bar, and the text
   "Hello, pixels!".  No framework involved — just direct iGui
   calls.  Runs interactively with:
       newcp-driver run-igui HelloPixels.Run
   Close the frame to exit. *)

IMPORT iGui, Console;

PROCEDURE Run*;
    VAR ok: INTSHORT;
        childId: INTEGER;
        kind, ec, timeMs, p1, p2, p3, p4: INTEGER;
        title: ARRAY 64 OF SHORTCHAR;
        text:  ARRAY 64 OF SHORTCHAR;
        family: ARRAY 32 OF SHORTCHAR;
        locale: ARRAY 8 OF SHORTCHAR;
BEGIN
    Console.WriteShortString("HelloPixels: opening child..."); Console.WriteLn;
    title := "Hello, pixels!";
    ok := iGui.OpenChild(title, childId);
    Console.WriteShortString("HelloPixels: OpenChild = "); Console.WriteInt(ok);
    Console.WriteShortString(" id="); Console.WriteInt(childId); Console.WriteLn;
    IF ok = 0 THEN RETURN END;

    (* Build one batch with three paint commands. *)
    iGui.BeginBatch(childId);
    (* Clear to white. *)
    iGui.EmitClear(1.0, 1.0, 1.0, 1.0);
    (* Black bar across the top. *)
    iGui.EmitFillRect(0.0, 0.0, 800.0, 50.0, 0.0,
                      0.0, 0.0, 0.0, 1.0);
    (* The message itself, in black, just below the bar. *)
    text   := "Hello, pixels!";
    family := "Segoe UI";
    locale := "en-us";
    iGui.EmitDrawTextRun(text, 20.0, 70.0, 32.0, family,
                          400, 0, 5, locale, -1.0, 0, 0,
                          0.0, 0.0, 0.0, 1.0);
    ok := iGui.SubmitBatch();
    Console.WriteShortString("HelloPixels: SubmitBatch = "); Console.WriteInt(ok); Console.WriteLn;

    (* Pump events until the user closes the frame. *)
    REPEAT
        ok := iGui.NextEvent(kind, ec, timeMs, p1, p2, p3, p4, -1);
        IF (ok # 0) & (kind = iGui.EvFrameClose) THEN EXIT END
    UNTIL FALSE
END Run;

END HelloPixels.
