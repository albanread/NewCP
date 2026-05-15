MODULE BrkProbe;
(* Smoke test for the BRK statement.  Calling BrkProbe.Run should
   dump heap / register / stack-walk to stderr and return 42. *)

PROCEDURE Run* (): INTEGER;
    VAR x: INTEGER;
BEGIN
    x := 7;
    BRK;            (* snapshot point *)
    x := x * 6;
    RETURN x
END Run;

END BrkProbe.
