MODULE M_Expr_LEN_OnString;
    PROCEDURE Measure (IN s: ARRAY OF CHAR): INTEGER;
    BEGIN RETURN LEN(s) END Measure;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        (* "abc" → 3 chars + NUL = 4 *)
        RETURN Measure("abc")
    END Run;
END M_Expr_LEN_OnString.
