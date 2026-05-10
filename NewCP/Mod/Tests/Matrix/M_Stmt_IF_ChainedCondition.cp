MODULE M_Stmt_IF_ChainedCondition;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: INTEGER;
    BEGIN
        a := 1; b := 2; c := 3;
        IF (a < b) & (b < c) & (a < c) THEN
            RETURN 1
        ELSE
            RETURN 0
        END
    END Run;
END M_Stmt_IF_ChainedCondition.
