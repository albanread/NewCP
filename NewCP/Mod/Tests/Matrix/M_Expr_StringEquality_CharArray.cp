MODULE M_Expr_StringEquality_CharArray;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: ARRAY 8 OF CHAR;
    BEGIN
        a := "hello";
        b := "hello";
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_StringEquality_CharArray.
