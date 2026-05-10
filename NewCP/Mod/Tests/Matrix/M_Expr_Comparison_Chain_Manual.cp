MODULE M_Expr_Comparison_Chain_Manual;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: INTEGER;
    BEGIN
        a := 1; b := 5; c := 10;
        IF (a < b) & (b < c) THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_Comparison_Chain_Manual.
