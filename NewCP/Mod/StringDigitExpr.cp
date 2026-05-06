MODULE StringDigitExpr;

IMPORT Console;

PROCEDURE Run*;
  VAR
    buf: ARRAY 8 OF SHORTCHAR;
    n: INTEGER;
BEGIN
  n := 42;
  buf[0] := SHORT(CHR(ORD('0') + n MOD 10));
  buf[1] := SHORT(CHR(ORD('0') + n DIV 10));
  buf[2] := 0X;
  Console.WriteInt(ORD(buf[0])); Console.WriteLn;
  Console.WriteInt(ORD(buf[1])); Console.WriteLn;
  Console.WriteShortString(buf); Console.WriteLn
END Run;

END StringDigitExpr.