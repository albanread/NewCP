MODULE OutProbe;
(* Smoke test for the BB-faithful Out module.  Exercises every
   public proc; output goes to Console (since we route Out through
   Console.cp in this slice).  Returns 1 if all six emit paths
   ran without trapping. *)

IMPORT Out;

PROCEDURE Run* (): INTEGER;
BEGIN
    Out.Open;
    Out.String("hello, ");
    Out.String("Out!");
    Out.Ln;
    Out.Int(42, 6);
    Out.Char(" ");
    Out.Int(-17, 0);
    Out.Ln;
    Out.Real(3.14, 0);
    Out.Ln;
    RETURN 1
END Run;

END OutProbe.
