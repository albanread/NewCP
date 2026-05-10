MODULE M_Module_VAR_Shared;
    VAR counter: INTEGER;

    PROCEDURE Bump (k: INTEGER);
    BEGIN counter := counter + k END Bump;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        counter := 0;
        Bump(7);
        Bump(11);
        Bump(12);
        RETURN counter
    END Run;
END M_Module_VAR_Shared.
