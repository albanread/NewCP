MODULE M_Param_IN_OpenArray_LengthSurvives;
    PROCEDURE Sum (IN p: ARRAY OF INTEGER): INTEGER;
        VAR i, s: INTEGER;
    BEGIN
        s := 0; i := 0;
        WHILE i < LEN(p) DO s := s + p[i]; INC(i) END;
        RETURN s
    END Sum;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 3; a[1] := 3; a[2] := 3; a[3] := 3;
        RETURN Sum(a)
    END Run;
END M_Param_IN_OpenArray_LengthSurvives.
