MODULE M_Method_OnRecord_FromExternalCallable;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;
        Make    = PROCEDURE (): Box;

    PROCEDURE (b: Box) Times* (k: INTEGER): INTEGER, NEW;
    BEGIN RETURN b.v * k END Times;

    PROCEDURE FreshFive (): Box;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := 5;
        RETURN b
    END FreshFive;

    PROCEDURE Run* (): INTEGER;
        VAR maker: Make; b: Box;
    BEGIN
        maker := FreshFive;
        b := maker();
        RETURN b.Times(5)                     (* 25 *)
    END Run;
END M_Method_OnRecord_FromExternalCallable.
