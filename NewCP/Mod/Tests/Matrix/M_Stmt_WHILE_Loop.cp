MODULE M_Stmt_WHILE_Loop;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum: INTEGER;
    BEGIN
        i := 1; sum := 0;
        WHILE i <= 10 DO
            sum := sum + i;
            INC(i)
        END;
        RETURN sum                          (* 1+2+...+10 = 55 *)
    END Run;
END M_Stmt_WHILE_Loop.
