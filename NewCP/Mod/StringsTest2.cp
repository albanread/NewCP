MODULE StringsTest2;
IMPORT Console;

PROCEDURE CopyStr(VAR dst: ARRAY OF SHORTCHAR; dstCap: INTEGER; src: ARRAY OF SHORTCHAR);
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (src[i] # 0X) & (i < dstCap - 1) DO
    dst[i] := src[i]; INC(i)
  END;
  dst[i] := 0X
END CopyStr;

PROCEDURE Run*;
  VAR
    title: ARRAY 64 OF SHORTCHAR;
BEGIN
  CopyStr(title, 64, "NewCP Log");
  Console.WriteShortString("title=["); Console.WriteShortString(title);
  Console.WriteShortString("]"); Console.WriteLn
END Run;

END StringsTest2.
