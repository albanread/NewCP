MODULE StringIntInspect;

IMPORT Console;

PROCEDURE IntToStr(n: INTEGER; VAR buf: ARRAY OF SHORTCHAR): INTEGER;
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    i, j: INTEGER;
BEGIN
  i := 23; digits[i] := 0X;
  IF n = 0 THEN
    DEC(i); digits[i] := 30X
  ELSE
    WHILE n > 0 DO
      DEC(i);
      digits[i] := SHORT(CHR(ORD('0') + n MOD 10));
      n := n DIV 10
    END
  END;
  j := 0;
  WHILE digits[i] # 0X DO
    buf[j] := digits[i]; INC(i); INC(j)
  END;
  buf[j] := 0X;
  RETURN j
END IntToStr;

PROCEDURE Run*;
  VAR buf: ARRAY 32 OF SHORTCHAR;
BEGIN
  IntToStr(42, buf);
  Console.WriteInt(ORD(buf[0])); Console.WriteLn;
  Console.WriteInt(ORD(buf[1])); Console.WriteLn;
  Console.WriteInt(ORD(buf[2])); Console.WriteLn
END Run;

END StringIntInspect.