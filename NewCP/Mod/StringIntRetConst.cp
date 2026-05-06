MODULE StringIntRetConst;

IMPORT Console;

PROCEDURE Fill42(VAR buf: ARRAY OF SHORTCHAR): INTEGER;
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    n, i, j: INTEGER;
BEGIN
  n := 42;
  i := 23; digits[i] := 0X;
  WHILE n > 0 DO
    DEC(i);
    digits[i] := SHORT(CHR(ORD('0') + n MOD 10));
    n := n DIV 10
  END;
  j := 0;
  WHILE digits[i] # 0X DO
    buf[j] := digits[i];
    INC(i); INC(j)
  END;
  buf[j] := 0X;
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

END StringIntRetConst.