MODULE M_Proc_Returns_SET;
    PROCEDURE Build (): SET;
        VAR s: SET;
    BEGIN
        s := {2, 4, 6};
        RETURN s
    END Build;

    PROCEDURE Run* (): INTEGER;
        VAR s: SET;
    BEGIN
        s := Build();
        IF (4 IN s) & ~(3 IN s) THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Proc_Returns_SET.
