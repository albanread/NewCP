MODULE M_Stmt_FOR_WithDecreasingRange;
    PROCEDURE Run* (): INTEGER;
        VAR i, count: INTEGER;
    BEGIN
        count := 0;
        FOR i := 10 TO 5 DO INC(count) END;     (* default step +1; 10 > 5 → no iters *)
        RETURN count
    END Run;
END M_Stmt_FOR_WithDecreasingRange.
