MODULE PtrVarArgProbe;
(* Repro for: pass a POINTER TO Record local as a VAR Record
   argument.  The callee should mutate the heap record, and
   the caller should see the change. *)

TYPE
    R = RECORD x: INTEGER END;
    P = POINTER TO R;

PROCEDURE Mutate (VAR r: R);
BEGIN r.x := 42 END Mutate;

PROCEDURE Run* (): INTEGER;
    VAR p: P;
BEGIN
    NEW(p);
    p.x := 0;
    Mutate(p^);
    RETURN p.x
END Run;

END PtrVarArgProbe.
