MODULE M_Procedure_LongParameterList;
    PROCEDURE Sum7 (a, b, c, d, e, f, g: INTEGER): INTEGER;
    BEGIN
        RETURN a + b + c + d + e + f + g
    END Sum7;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN Sum7(1, 2, 3, 4, 5, 6, 7)   (* 28 *)
    END Run;
END M_Procedure_LongParameterList.
