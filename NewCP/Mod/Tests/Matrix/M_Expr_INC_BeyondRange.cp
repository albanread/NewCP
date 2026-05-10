MODULE M_Expr_INC_BeyondRange;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 100000;
        INC(n, 1000000);
        RETURN n
    END Run;
END M_Expr_INC_BeyondRange.
