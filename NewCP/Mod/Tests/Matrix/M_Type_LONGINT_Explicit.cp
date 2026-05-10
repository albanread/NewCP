MODULE M_Type_LONGINT_Explicit;
    PROCEDURE Run* (): LONGINT;
        VAR a, b: LONGINT;
    BEGIN
        a := 10000000000;        (* > 2^31 *)
        b := 20000000000;
        (* (b - a) DIV 1_000_000_000 = 10_000_000_000 / 1_000_000_000 = 10 *)
        RETURN (b - a) DIV 1000000000
    END Run;
END M_Type_LONGINT_Explicit.
