MODULE M_Expr_BOOLEAN_FromComparison;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER; flag: BOOLEAN;
    BEGIN
        a := 5; b := 3;
        flag := a > b;
        IF flag THEN RETURN 7 ELSE RETURN 0 END
    END Run;
END M_Expr_BOOLEAN_FromComparison.
