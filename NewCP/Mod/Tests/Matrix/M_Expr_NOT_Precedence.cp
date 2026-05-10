MODULE M_Expr_NOT_Precedence;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: BOOLEAN;
    BEGIN
        a := FALSE; b := TRUE;
        (* (~a) & b = TRUE & TRUE = TRUE *)
        IF ~a & b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_NOT_Precedence.
