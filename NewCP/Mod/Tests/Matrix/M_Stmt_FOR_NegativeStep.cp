MODULE M_Stmt_FOR_NegativeStep;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        sum := 0;
        FOR i := 5 TO 1 BY -1 DO sum := sum + i END;
        RETURN sum                      (* 5+4+3+2+1 = 15 *)
    END Run;
END M_Stmt_FOR_NegativeStep.
