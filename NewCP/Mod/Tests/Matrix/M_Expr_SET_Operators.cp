MODULE M_Expr_SET_Operators;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, u, i, d, sd: SET; score: INTEGER;
    BEGIN
        a := {0, 1, 2};
        b := {1, 2, 3};
        u  := a + b;                 (* {0,1,2,3} *)
        i  := a * b;                 (* {1,2}     *)
        d  := a - b;                 (* {0}       *)
        sd := a / b;                 (* {0,3}     *)
        score := 0;
        IF (0 IN u) & (3 IN u) & (1 IN u) & (2 IN u) THEN score := score + 1 END;
        IF (1 IN i) & (2 IN i) & ~(0 IN i) & ~(3 IN i) THEN score := score + 20 END;
        IF (0 IN d) & ~(1 IN d) & ~(2 IN d) & ~(3 IN d) THEN score := score + 300 END;
        IF (0 IN sd) & (3 IN sd) & ~(1 IN sd) & ~(2 IN sd) THEN score := score + 4000 END;
        RETURN score
    END Run;
END M_Expr_SET_Operators.
