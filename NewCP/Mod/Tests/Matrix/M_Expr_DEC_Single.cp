MODULE M_Expr_DEC_Single;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 10;
        DEC(n);
        RETURN n
    END Run;
END M_Expr_DEC_Single.
