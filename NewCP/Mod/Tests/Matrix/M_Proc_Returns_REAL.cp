MODULE M_Proc_Returns_REAL;
    PROCEDURE Compute (n: INTEGER): REAL;
    BEGIN RETURN n * 1.5 END Compute;

    PROCEDURE Run* (): LONGINT;
        VAR r: REAL;
    BEGIN
        r := Compute(8);
        RETURN ENTIER(r)
    END Run;
END M_Proc_Returns_REAL.
