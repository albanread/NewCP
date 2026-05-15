MODULE VarRefChainProbe;
(* Repro for: VAR-receiver method calls a top-level VAR-param
   procedure, passing the receiver `s`.  The procedure mutates
   the field — does the mutation survive back at the call site? *)

TYPE
    R = RECORD x: INTEGER END;

PROCEDURE Mutate (VAR r: R);
BEGIN r.x := 42 END Mutate;

PROCEDURE (VAR self: R) Method, NEW;
BEGIN
    Mutate(self)
END Method;

PROCEDURE Run* (): INTEGER;
    VAR r: R;
BEGIN
    r.x := 0;
    r.Method;
    RETURN r.x
END Run;

END VarRefChainProbe.
