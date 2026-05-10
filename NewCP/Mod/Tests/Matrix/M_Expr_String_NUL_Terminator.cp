MODULE M_Expr_String_NUL_Terminator;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: ARRAY 8 OF CHAR;
    BEGIN
        a[0] := "h"; a[1] := "i"; a[2] := 0X; a[3] := "X"; a[4] := 0X;
        b := "hi";
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_String_NUL_Terminator.
