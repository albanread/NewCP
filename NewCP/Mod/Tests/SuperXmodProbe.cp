MODULE SuperXmodProbe;
(* Cross-module super call: ChildDesc.Bump (here) chains via
   `c.Bump^(v)` into SuperBase.BaseDesc.Bump (in another module).
   Verifies that the super-call lowering correctly emits an
   ImportRef-style direct call to the base module's method. *)

    IMPORT SuperBase;

    TYPE
        ChildDesc* = RECORD (SuperBase.BaseDesc)
            traceChild*: INTEGER
        END;
        Child* = POINTER TO ChildDesc;

    PROCEDURE (c: ChildDesc) Bump* (v: INTEGER);
    BEGIN
        c.Bump^(v);                  (* super: cross-module BaseDesc.Bump *)
        c.traceChild := c.traceChild + (v * 10)
    END Bump;

    PROCEDURE Run* (): INTEGER;
        VAR c: Child;
    BEGIN
        NEW(c);
        c.Bump(3);
        RETURN (c.traceBase * 100) + c.traceChild
    END Run;

END SuperXmodProbe.
