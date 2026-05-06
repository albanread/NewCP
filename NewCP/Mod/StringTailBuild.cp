MODULE StringTailBuild;

IMPORT Console;

PROCEDURE Build42(VAR buf: ARRAY OF SHORTCHAR);
  VAR
    digits: ARRAY 24 OF SHORTCHAR;
    i, j: INTEGER;
BEGIN
  i := 23;
  digits[i] := 0X;
  DEC(i); digits[i] := 32X;
  DEC(i); digits[i] := 34X;
  j := 0;
  WHILE digits[i] # 0X DO
    buf[j] := digits[i];
    INC(i); INC(j)
  END;
  buf[j] := 0X
END Build42;

PROCEDURE Run*;
  VAR buf: ARRAY 32 OF SHORTCHAR;
BEGIN
  Build42(buf);
  Console.WriteInt(ORD(buf[0])); Console.WriteLn;
  Console.WriteInt(ORD(buf[1])); Console.WriteLn;
  Console.WriteInt(ORD(buf[2])); Console.WriteLn;
  Console.WriteShortString(buf); Console.WriteLn
END Run;

END StringTailBuild.