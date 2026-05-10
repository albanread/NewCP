MODULE M_Method_OnPointerAlias_AbstractBase_ConcreteSub;
    TYPE
        BaseDesc = ABSTRACT RECORD END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) v: INTEGER END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Eval* (n: INTEGER): INTEGER, NEW, ABSTRACT;

    PROCEDURE (s: Sub) Eval* (n: INTEGER): INTEGER;
    BEGIN RETURN s.v * n END Eval;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s);
        s.v := 12;
        RETURN s.Eval(12)                       (* 144 *)
    END Run;
END M_Method_OnPointerAlias_AbstractBase_ConcreteSub.
