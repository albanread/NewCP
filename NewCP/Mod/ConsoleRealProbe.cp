MODULE ConsoleRealProbe;

IMPORT Console;

PROCEDURE Run*;
  VAR x: REAL;
BEGIN
  Console.WriteShortString("ConsoleRealProbe begin");
  Console.WriteLn;

  Console.WriteReal(0.25);
  Console.WriteLn;

  Console.WriteReal(1.0);
  Console.WriteLn;

  Console.WriteReal(-3.5);
  Console.WriteLn;

  x := 2.0;
  x := x + 0.5;
  Console.WriteReal(x);
  Console.WriteLn;

  Console.WriteShortString("ConsoleRealProbe end");
  Console.WriteLn
END Run;

END ConsoleRealProbe.