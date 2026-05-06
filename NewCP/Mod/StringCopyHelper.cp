MODULE StringCopyHelper;

IMPORT Console;

PROCEDURE Fill42(VAR buf: ARRAY OF SHORTCHAR);
  VAR digits: ARRAY 24 OF SHORTCHAR;
      i, j: INTEGER;
BEGIN
  digits[0] := 34X;
  digits[1] := 32X;
  digits[2] := 0X;
  i := 0;
  j := 0;
  WHILE digits[i] # 0X DO
    buf[j] := digits[i];
    INC(i); INC(j)
  END;
  buf[j] := 0X
END Fill42;

PROCEDURE Run*;
  VAR buf: ARRAY 32 OF SHORTCHAR;
BEGIN
  Fill42(buf);
  Console.WriteInt(ORD(buf[0])); Console.WriteLn
END Run;

END StringCopyHelper.