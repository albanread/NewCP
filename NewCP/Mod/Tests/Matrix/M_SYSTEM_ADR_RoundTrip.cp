MODULE M_SYSTEM_ADR_RoundTrip;
    IMPORT SYSTEM;

    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER; a, b: INTEGER;
    BEGIN
        x := 42;
        a := SYSTEM.ADR(x);
        b := SYSTEM.ADR(x);
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_SYSTEM_ADR_RoundTrip.
