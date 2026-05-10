MODULE M_Expr_ABS_OnReal;
    PROCEDURE Run* (): LONGINT;
        VAR x: REAL;
    BEGIN
        x := -7.0;
        RETURN ENTIER(ABS(x))
    END Run;
END M_Expr_ABS_OnReal.
