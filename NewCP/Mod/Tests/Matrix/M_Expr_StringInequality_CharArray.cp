MODULE M_Expr_StringInequality_CharArray;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: ARRAY 8 OF CHAR;
    BEGIN
        a := "hello";
        b := "world";
        IF a # b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_StringInequality_CharArray.
