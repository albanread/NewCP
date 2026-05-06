MODULE StringRetHelper;

IMPORT Console;

PROCEDURE Fill42(VAR buf: ARRAY OF SHORTCHAR): INTEGER;
BEGIN
  buf[0] := 34X;
  buf[1] := 32X;
  buf[2] := 0X;
  RETURN 2
END Fill42;

PROCEDURE Run*;
  VAR
    buf: ARRAY 32 OF SHORTCHAR;
    n: INTEGER;
BEGIN
  n := Fill42(buf);
  Console.WriteInt(n); Console.WriteLn;
  Console.WriteShortString(buf); Console.WriteLn
END Run;

END StringRetHelper.