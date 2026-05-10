MODULE M_Stmt_RETURN_FromInside_WITH;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        SubDesc  = RECORD (BaseDesc) value: INTEGER END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE PullValue (p: Base): INTEGER;
    BEGIN
        WITH p: Sub DO
            RETURN p.value
        ELSE
            RETURN 0
        END
    END PullValue;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s); s.value := 77;
        RETURN PullValue(s)
    END Run;
END M_Stmt_RETURN_FromInside_WITH.
