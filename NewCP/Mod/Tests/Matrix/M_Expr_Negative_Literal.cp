MODULE M_Expr_Negative_Literal;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 10 + (-3);
        RETURN x
    END Run;
END M_Expr_Negative_Literal.
