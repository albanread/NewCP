MODULE StringsTest1;
IMPORT Console;

PROCEDURE Run*;
  VAR
    title: ARRAY 64 OF SHORTCHAR;
BEGIN
  title[0] := 0X;
  Console.WriteShortString("null check: [");
  IF title[0] = 0X THEN
    Console.WriteShortString("ok]")
  ELSE
    Console.WriteShortString("FAIL]")
  END;
  Console.WriteLn
END Run;

END StringsTest1.
