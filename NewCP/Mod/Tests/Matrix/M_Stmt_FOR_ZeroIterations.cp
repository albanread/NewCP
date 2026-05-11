MODULE M_Stmt_FOR_ZeroIterations;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := 5 TO 3 DO sum := sum + 999 END;
        RETURN sum
    END Run;
END M_Stmt_FOR_ZeroIterations.
