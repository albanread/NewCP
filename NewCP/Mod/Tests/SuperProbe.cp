MODULE SuperProbe;
(* Smoke test for super-call (`m.Method^(args)`) lowering. *)

    TYPE
        BaseDesc* = EXTENSIBLE RECORD
            traceBase*: INTEGER
        END;
        Base* = POINTER TO BaseDesc;

        ChildDesc* = RECORD (BaseDesc)
            traceChild*: INTEGER
        END;
        Child* = POINTER TO ChildDesc;

    PROCEDURE (b: BaseDesc) Bump* (v: INTEGER), NEW, EXTENSIBLE;
    BEGIN
        b.traceBase := b.traceBase + v
    END Bump;

    PROCEDURE (c: ChildDesc) Bump* (v: INTEGER);
    BEGIN
        c.Bump^(v);                  (* super: call BaseDesc.Bump *)
        c.traceChild := c.traceChild + (v * 10)
    END Bump;

    PROCEDURE Run* (): INTEGER;
        VAR c: Child;
    BEGIN
        NEW(c);
        c.Bump(3);
        RETURN (c.traceBase * 100) + c.traceChild
    END Run;

END SuperProbe.
