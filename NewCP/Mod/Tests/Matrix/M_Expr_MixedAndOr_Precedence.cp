MODULE M_Expr_MixedAndOr_Precedence;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: BOOLEAN;
    BEGIN
        a := TRUE; b := FALSE; c := FALSE;
        (* a OR (b & c) = TRUE OR (FALSE & FALSE) = TRUE *)
        IF a OR b & c THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_MixedAndOr_Precedence.
