MODULE CrossTest;
(**
   CrossTest — minimal cross-module call test.
   Imports ImportBase (CP→CP) and prints results via Console (CP→native).
*)

IMPORT Console, ImportBase, ImportUse;

PROCEDURE Run*;
  VAR r: INTEGER;
BEGIN
  (* CP → native *)
  Console.WriteShortString("CrossTest: starting"); Console.WriteLn;

  (* CP → CP (one hop) *)
  r := ImportBase.AddOne(10);
  Console.WriteShortString("AddOne(10) = "); Console.WriteInt(r); Console.WriteLn;

  (* CP → CP → CP (two hops) *)
  r := ImportUse.TwiceImported(5);
  Console.WriteShortString("TwiceImported(5) = "); Console.WriteInt(r); Console.WriteLn;

  Console.WriteShortString("CrossTest: done"); Console.WriteLn
END Run;

END CrossTest.
