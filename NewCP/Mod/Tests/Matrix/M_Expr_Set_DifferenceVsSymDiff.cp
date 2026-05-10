MODULE M_Expr_Set_DifferenceVsSymDiff;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, diff, sym: SET; score: INTEGER;
    BEGIN
        a := {1, 2, 3};
        b := {2, 3, 4};
        diff := a - b;                          (* {1} *)
        sym  := a / b;                          (* {1, 4} *)
        score := 0;
        IF (1 IN diff) & ~(4 IN diff) THEN score := score + 1  END;
        IF (1 IN sym) & (4 IN sym)    THEN score := score + 10 END;
        RETURN score
    END Run;
END M_Expr_Set_DifferenceVsSymDiff.
