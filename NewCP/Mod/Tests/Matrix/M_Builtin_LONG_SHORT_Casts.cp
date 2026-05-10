MODULE M_Builtin_LONG_SHORT_Casts;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER; l: LONGINT;
    BEGIN
        n := 250;
        l := LONG(n);
        n := SHORT(l);
        RETURN n
    END Run;
END M_Builtin_LONG_SHORT_Casts.
