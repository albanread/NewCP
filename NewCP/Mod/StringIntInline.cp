MODULE StringIntInline;

IMPORT Console;

PROCEDURE Run*;
  VAR
    buf: ARRAY 32 OF SHORTCHAR;
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
  Console.WriteInt(ORD(buf[0])); Console.WriteLn;
  Console.WriteInt(ORD(buf[1])); Console.WriteLn;
  Console.WriteInt(ORD(buf[2])); Console.WriteLn;
  Console.WriteShortString(buf); Console.WriteLn
END Run;

END StringIntInline.