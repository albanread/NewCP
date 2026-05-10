MODULE M_Stmt_REPEAT_ManyIters;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        i := 0; sum := 0;
        REPEAT
            INC(i);
            sum := sum + i
        UNTIL i = 10;
        RETURN sum                              (* 1+2+...+10 = 55 *)
    END Run;
END M_Stmt_REPEAT_ManyIters.
