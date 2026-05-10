MODULE M_Method_OnExtensible_NoOverride;
    TYPE
        BaseDesc = EXTENSIBLE RECORD v: INTEGER END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE (b: Base) Get* (): INTEGER, NEW, EXTENSIBLE;
    BEGIN RETURN b.v END Get;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s);
        s.v := 33;
        RETURN s.Get()
    END Run;
END M_Method_OnExtensible_NoOverride.
