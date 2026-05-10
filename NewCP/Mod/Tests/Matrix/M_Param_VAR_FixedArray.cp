MODULE M_Param_VAR_FixedArray;
    PROCEDURE Mutate (VAR a: ARRAY 4 OF INTEGER);
    BEGIN a[0] := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: ARRAY 4 OF INTEGER;
    BEGIN
        caller[0] := 42;
        Mutate(caller);
        RETURN caller[0]
    END Run;
END M_Param_VAR_FixedArray.
