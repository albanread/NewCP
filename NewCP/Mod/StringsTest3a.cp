MODULE StringsTest3a;
IMPORT Console;

PROCEDURE Fmt(n: INTEGER; VAR buf: ARRAY OF SHORTCHAR): INTEGER;
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    i, j: INTEGER;
BEGIN
  i := 23; digits[i] := 0X;
  IF n = 0 THEN
    DEC(i); digits[i] := 30X
  END;
  j := 0;
  WHILE digits[i] # 0X DO buf[j] := digits[i]; INC(i); INC(j) END;
  buf[j] := 0X;
  RETURN j
END Fmt;

PROCEDURE Run*;
  VAR
    buf: ARRAY 32 OF SHORTCHAR;
    n: INTEGER;
BEGIN
  n := Fmt(0, buf);
  Console.WriteShortString("got=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn
END Run;

END StringsTest3a.
