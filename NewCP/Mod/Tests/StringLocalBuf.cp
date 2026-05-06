MODULE StringLocalBuf;

IMPORT Console;

PROCEDURE Run*;
  VAR buf: ARRAY 8 OF SHORTCHAR;
BEGIN
  buf[0] := 41X;
  buf[1] := 0X;
  Console.WriteShortString("before"); Console.WriteLn;
  Console.WriteShortString(buf); Console.WriteLn;

  buf[0] := 41X;
  buf[1] := 42X;
  buf[2] := 0X;
  Console.WriteShortString(buf); Console.WriteLn;
  Console.WriteShortString("after"); Console.WriteLn
END Run;

END StringLocalBuf.