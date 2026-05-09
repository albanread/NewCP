MODULE Phase2EventDemo;

(* iGui Phase 2 acceptance demo. Loops on iGui.NextEvent and prints
   each event to the console until the user closes the frame.

   Run with:
     newcp-driver run-igui Phase2EventDemo.Run

   Type characters, click in the window, resize it, then close it.
   Each event prints a one-line summary; the loop exits cleanly when
   the EvFrameClose arrives. *)

IMPORT iGui, Console;

PROCEDURE WriteHexShort(n: INTEGER);
  VAR digits: ARRAY 9 OF SHORTCHAR;
      i, val: INTEGER;
      d: INTEGER;
BEGIN
  val := n;
  IF val < 0 THEN val := val + 65536 END;
  i := 7;
  WHILE i >= 0 DO
    d := val MOD 16;
    IF d < 10 THEN
      digits[i] := SHORT(CHR(d + ORD("0")))
    ELSE
      digits[i] := SHORT(CHR(d - 10 + ORD("A")))
    END;
    val := val DIV 16;
    DEC(i)
  END;
  digits[8] := 0X;
  Console.WriteShortString(digits)
END WriteHexShort;

PROCEDURE Run*;
  VAR
    kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
    ok: INTSHORT;
    op: INTEGER;
BEGIN
  Console.WriteShortString("Phase 2 event demo: type, click, resize. Close window to exit.");
  Console.WriteLn;

  REPEAT
    ok := iGui.NextEvent(kind, childId, timeMs, p1, p2, p3, p4, -1);
    IF ok # 0 THEN
      IF kind = iGui.EvKey THEN
        Console.WriteShortString("[key] vkey=0x"); WriteHexShort(p1);
        Console.WriteShortString(" scancode="); Console.WriteInt(p2);
        Console.WriteShortString(" mods="); Console.WriteInt(p3);
        Console.WriteShortString(" down="); Console.WriteInt(p4 MOD 65536);
        Console.WriteShortString(" t="); Console.WriteInt(timeMs);
        Console.WriteLn
      ELSIF kind = iGui.EvChar THEN
        Console.WriteShortString("[char] cp="); Console.WriteInt(p1);
        Console.WriteShortString(" mods="); Console.WriteInt(p2);
        Console.WriteLn
      ELSIF kind = iGui.EvMouse THEN
        op := p3 DIV 65536;
        Console.WriteShortString("[mouse] op="); Console.WriteInt(op);
        Console.WriteShortString(" x="); Console.WriteInt(p1);
        Console.WriteShortString(" y="); Console.WriteInt(p2);
        IF op = iGui.MouseWheel THEN
          Console.WriteShortString(" delta="); Console.WriteInt(p4 MOD 65536);
          Console.WriteShortString(" lines="); Console.WriteInt(p4 DIV 65536)
        END;
        Console.WriteLn
      ELSIF kind = iGui.EvResize THEN
        Console.WriteShortString("[resize] "); Console.WriteInt(p1);
        Console.WriteShortString("x"); Console.WriteInt(p2); Console.WriteLn
      ELSIF kind = iGui.EvFocus THEN
        IF p1 # 0 THEN
          Console.WriteShortString("[focus] gained"); Console.WriteLn
        ELSE
          Console.WriteShortString("[focus] lost"); Console.WriteLn
        END
      ELSIF kind = iGui.EvFrameClose THEN
        Console.WriteShortString("[frame-close]"); Console.WriteLn;
        EXIT
      END
    END
  UNTIL FALSE
END Run;

END Phase2EventDemo.
