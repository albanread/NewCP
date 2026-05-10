MODULE M_Stmt_IS_Inside_WITH;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        MidDesc  = EXTENSIBLE RECORD (BaseDesc) END;
        Mid      = POINTER TO MidDesc;
        SubDesc  = RECORD (MidDesc) v: INTEGER END;
        Sub      = POINTER TO SubDesc;

    PROCEDURE Inspect (p: Base): INTEGER;
    BEGIN
        WITH p: Mid DO
            IF p IS Sub THEN
                RETURN p(Sub).v
            ELSE
                RETURN -1
            END
        ELSE
            RETURN -2
        END
    END Inspect;

    PROCEDURE Run* (): INTEGER;
        VAR s: Sub;
    BEGIN
        NEW(s); s.v := 99;
        RETURN Inspect(s)
    END Run;
END M_Stmt_IS_Inside_WITH.
