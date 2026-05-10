MODULE M_Stmt_WITH_MultiArm;
    TYPE
        BaseDesc = EXTENSIBLE RECORD tag: INTEGER END;
        Base     = POINTER TO BaseDesc;
        ADesc    = RECORD (BaseDesc) av: INTEGER END;
        A        = POINTER TO ADesc;
        BDesc    = RECORD (BaseDesc) bv: INTEGER END;
        B        = POINTER TO BDesc;

    PROCEDURE Score (p: Base): INTEGER;
    BEGIN
        WITH p: A DO
            RETURN 10 + p.av
        |  p: B DO
            RETURN 20 + p.bv
        ELSE
            RETURN 100
        END
    END Score;

    PROCEDURE Run* (): INTEGER;
        VAR a: A; b: B;
    BEGIN
        NEW(a); a.av := 1;
        NEW(b); b.bv := 2;
        (* Score(a) = 10 + 1 = 11; Score(b) = 20 + 2 = 22; sum = 33 *)
        RETURN Score(a) + Score(b)
    END Run;
END M_Stmt_WITH_MultiArm.
