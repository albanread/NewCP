MODULE M_RecvPtrAlias_NewOnAbstract;
    TYPE
        BaseDesc* = ABSTRACT RECORD tag*: INTEGER END;
        Base*     = POINTER TO BaseDesc;
        SubDesc*  = RECORD (BaseDesc) extra*: INTEGER END;
        Sub*      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Greet* (v: INTEGER), NEW, ABSTRACT;

    PROCEDURE (s: Sub) Greet* (v: INTEGER);
    BEGIN s.tag := v; s.extra := v * 10 END Greet;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s); s.Greet(7);
        RETURN (s.tag * 100) + s.extra
    END Run;
END M_RecvPtrAlias_NewOnAbstract.
