MODULE M_Type_INTSHORT_Roundtrip;
    PROCEDURE Pass (x: INTSHORT): INTSHORT;
    BEGIN RETURN x END Pass;

    PROCEDURE Run* (): INTEGER;
        VAR n: INTSHORT;
    BEGIN
        n := 32000;
        n := Pass(n);
        RETURN n
    END Run;
END M_Type_INTSHORT_Roundtrip.
