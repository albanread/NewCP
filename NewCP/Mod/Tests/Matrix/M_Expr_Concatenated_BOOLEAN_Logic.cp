MODULE M_Expr_Concatenated_BOOLEAN_Logic;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c, d: BOOLEAN; result: BOOLEAN;
    BEGIN
        a := TRUE; b := FALSE; c := TRUE; d := FALSE;
        (* (a & ~b) OR (c & d) = (TRUE & TRUE) OR (TRUE & FALSE) = TRUE *)
        result := (a & ~b) OR (c & d);
        IF result THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_Concatenated_BOOLEAN_Logic.
