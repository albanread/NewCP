MODULE M_Method_SuperCall_ThreeLevels;
    TYPE
        ADesc* = EXTENSIBLE RECORD END;
        A*     = POINTER TO ADesc;
        BDesc* = EXTENSIBLE RECORD (ADesc) END;
        B*     = POINTER TO BDesc;
        CDesc* = EXTENSIBLE RECORD (BDesc) END;
        C*     = POINTER TO CDesc;
        DDesc* = RECORD (CDesc) END;
        D*     = POINTER TO DDesc;

    PROCEDURE (a: A) Tag* (): INTEGER, NEW, EXTENSIBLE;
    BEGIN RETURN 1 END Tag;

    PROCEDURE (b: B) Tag* (): INTEGER, EXTENSIBLE;
    BEGIN RETURN b.Tag^() + 10 END Tag;

    PROCEDURE (c: C) Tag* (): INTEGER, EXTENSIBLE;
    BEGIN RETURN c.Tag^() + 100 END Tag;

    PROCEDURE (d: D) Tag* (): INTEGER;
    BEGIN RETURN d.Tag^() + 1000 END Tag;

    PROCEDURE Run* (): INTEGER;
        VAR d: D;
    BEGIN
        NEW(d);
        RETURN d.Tag()                            (* 1 + 10 + 100 + 1000 = 1111 *)
    END Run;
END M_Method_SuperCall_ThreeLevels.
