MODULE M_Stmt_For_StepOf_One;
    PROCEDURE Run* (): INTEGER;
        VAR i, n: INTEGER;
    BEGIN
        n := 0;
        FOR i := 1 TO 4 DO n := n + i END;   (* 1+2+3+4 = 10 *)
        RETURN n
    END Run;
END M_Stmt_For_StepOf_One.
