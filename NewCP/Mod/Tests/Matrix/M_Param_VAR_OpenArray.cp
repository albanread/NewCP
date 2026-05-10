MODULE M_Param_VAR_OpenArray;
    PROCEDURE Mutate (VAR p: ARRAY OF INTEGER);
    BEGIN p[0] := 999 END Mutate;

    PROCEDURE Run* (): INTEGER;
        VAR caller: ARRAY 4 OF INTEGER;
    BEGIN
        caller[0] := 42;
        Mutate(caller);
        RETURN caller[0]
    END Run;
END M_Param_VAR_OpenArray.
