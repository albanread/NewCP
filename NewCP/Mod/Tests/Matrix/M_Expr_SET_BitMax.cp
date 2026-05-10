MODULE M_Expr_SET_BitMax;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET;
    BEGIN
        s := {0, 31};
        IF (0 IN s) & (31 IN s) THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_SET_BitMax.
