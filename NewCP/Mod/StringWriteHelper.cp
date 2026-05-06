MODULE StringWriteHelper;

IMPORT Console;

PROCEDURE Fill42(VAR buf: ARRAY OF SHORTCHAR);
BEGIN
  buf[0] := 34X;
  buf[1] := 32X;
  buf[2] := 0X
END Fill42;

PROCEDURE Run*;
  VAR buf: ARRAY 32 OF SHORTCHAR;
BEGIN
  Fill42(buf);
  Console.WriteInt(ORD(buf[0])); Console.WriteLn;
  Console.WriteInt(ORD(buf[1])); Console.WriteLn;
  Console.WriteInt(ORD(buf[2])); Console.WriteLn;
  Console.WriteShortString(buf); Console.WriteLn
END Run;

END StringWriteHelper.