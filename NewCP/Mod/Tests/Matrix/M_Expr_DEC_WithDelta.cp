MODULE M_Expr_DEC_WithDelta;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 100;
        DEC(n, 85);
        RETURN n
    END Run;
END M_Expr_DEC_WithDelta.
