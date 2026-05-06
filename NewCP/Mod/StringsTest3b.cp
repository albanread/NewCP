MODULE StringsTest3b;
IMPORT Console;

PROCEDURE Run*;
  VAR
    buf: ARRAY 32 OF SHORTCHAR;
BEGIN
  buf[0] := 30X;
  buf[1] := 0X;
  Console.WriteShortString("got=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn
END Run;

END StringsTest3b.
