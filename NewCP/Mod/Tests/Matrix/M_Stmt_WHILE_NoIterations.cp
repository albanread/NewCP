MODULE M_Stmt_WHILE_NoIterations;
    PROCEDURE Run* (): INTEGER;
        VAR i, count: INTEGER;
    BEGIN
        i := 10; count := 0;
        WHILE i < 5 DO INC(count); INC(i) END;
        RETURN count
    END Run;
END M_Stmt_WHILE_NoIterations.
