MODULE StringsTest3;
IMPORT Console;

PROCEDURE IntToStr(n: INTEGER; VAR buf: ARRAY OF SHORTCHAR): INTEGER;
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    i, j: INTEGER;
    neg: BOOLEAN;
BEGIN
  i := 23; digits[i] := 0X;
  neg := n < 0;
  IF n = 0 THEN
    DEC(i); digits[i] := 30X
  ELSE
    IF neg THEN n := -n END;
    WHILE n > 0 DO
      DEC(i);
      digits[i] := SHORT(CHR(ORD('0') + n MOD 10));
      n := n DIV 10
    END;
    IF neg THEN DEC(i); digits[i] := 2DX END
  END;
  j := 0;
  WHILE digits[i] # 0X DO buf[j] := digits[i]; INC(i); INC(j) END;
  buf[j] := 0X;
  RETURN j
END IntToStr;

PROCEDURE Run*;
  VAR
    buf: ARRAY 32 OF SHORTCHAR;
    n: INTEGER;
BEGIN
  n := IntToStr(0, buf);
  Console.WriteShortString("IntToStr(0)=["); Console.WriteShortString(buf);
  Console.WriteShortString("]"); Console.WriteLn
END Run;

END StringsTest3.
