MODULE M_Type_ANYREC_Param;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        ADesc    = RECORD (BaseDesc) END;
        A        = POINTER TO ADesc;
        BDesc    = RECORD (BaseDesc) END;
        B        = POINTER TO BDesc;

    PROCEDURE Inspect (p: Base): INTEGER;
    BEGIN
        IF p IS A THEN RETURN 11 END;
        IF p IS B THEN RETURN 22 END;
        RETURN 0
    END Inspect;

    PROCEDURE Run* (): INTEGER;
        VAR b: B; bp: Base;
    BEGIN
        NEW(b);
        bp := b;
        RETURN Inspect(bp)
    END Run;
END M_Type_ANYREC_Param.
