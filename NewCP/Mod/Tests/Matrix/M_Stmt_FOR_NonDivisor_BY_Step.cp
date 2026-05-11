MODULE M_Stmt_FOR_NonDivisor_BY_Step;
    PROCEDURE Run* (): INTEGER;
        VAR i, sum, count: INTEGER;
    BEGIN
        sum := 0; count := 0;
        FOR i := 0 TO 10 BY 3 DO sum := sum + i; INC(count) END;
        (* iterates i = 0, 3, 6, 9.  sum = 18, count = 4 *)
        RETURN sum * 10 + count                       (* 184 *)
    END Run;
END M_Stmt_FOR_NonDivisor_BY_Step.
