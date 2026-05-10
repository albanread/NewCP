MODULE M_Nested_OpenArrayUpvalue;
    PROCEDURE Outer (IN p: ARRAY OF INTEGER): INTEGER;
        VAR result: INTEGER;

        PROCEDURE Inner (q: ARRAY OF INTEGER): INTEGER;
            VAR i, s: INTEGER;
        BEGIN
            s := 0; i := 0;
            WHILE i < LEN(q) DO s := s + q[i]; INC(i) END;
            RETURN s
        END Inner;

    BEGIN
        result := Inner(p);
        RETURN result
    END Outer;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 10; a[1] := 11; a[2] := 9; a[3] := 12;
        RETURN Outer(a)
    END Run;
END M_Nested_OpenArrayUpvalue.
