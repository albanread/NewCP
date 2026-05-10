MODULE M_Stmt_WITH_ElseOnly;
    TYPE
        BaseDesc = EXTENSIBLE RECORD END;
        Base     = POINTER TO BaseDesc;
        ADesc    = RECORD (BaseDesc) v: INTEGER END;
        A        = POINTER TO ADesc;
        UnusedDesc = RECORD (BaseDesc) END;
        Unused     = POINTER TO UnusedDesc;

    PROCEDURE Score (p: Base): INTEGER;
    BEGIN
        WITH p: Unused DO
            RETURN 1
        ELSE
            RETURN 999
        END
    END Score;

    PROCEDURE Run* (): INTEGER;
        VAR a: A;
    BEGIN
        NEW(a); a.v := 1;
        RETURN Score(a)         (* dynamic type is A, not Unused → ELSE *)
    END Run;
END M_Stmt_WITH_ElseOnly.
