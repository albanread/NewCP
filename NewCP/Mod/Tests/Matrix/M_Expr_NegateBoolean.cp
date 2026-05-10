MODULE M_Expr_NegateBoolean;
    PROCEDURE Run* (): INTEGER;
        VAR b: BOOLEAN;
    BEGIN
        b := TRUE;
        IF ~~b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_NegateBoolean.
