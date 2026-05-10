MODULE M_Param_Value_OpenArray;
    PROCEDURE Mutate (p: ARRAY OF INTEGER);
    BEGIN p[0] := 99 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR a: ARRAY 4 OF INTEGER;
    BEGIN
        a[0] := 7;
        Mutate(a);
        RETURN a[0]
    END Run;
END M_Param_Value_OpenArray.
