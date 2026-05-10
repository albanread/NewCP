MODULE M_Stmt_FOR_PositiveStep;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := 1 TO 9 BY 2 DO sum := sum + i END;
        RETURN sum                      (* 1 + 3 + 5 + 7 + 9 = 25 *)
    END Run;
END M_Stmt_FOR_PositiveStep.
