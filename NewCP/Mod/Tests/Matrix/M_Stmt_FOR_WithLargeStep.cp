MODULE M_Stmt_FOR_WithLargeStep;
    PROCEDURE Run* (): INTEGER;
        VAR i, count: INTEGER;
    BEGIN
        count := 0;
        FOR i := 0 TO 5 BY 100 DO INC(count) END;
        RETURN count                          (* 1 iteration: i=0 *)
    END Run;
END M_Stmt_FOR_WithLargeStep.
