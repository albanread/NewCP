MODULE M_Method_With_LocalVar;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) PowerOf* (k: INTEGER): INTEGER, NEW;
        VAR i, result: INTEGER;
    BEGIN
        result := 1;
        FOR i := 1 TO k DO result := result * b.v END;
        RETURN result
    END PowerOf;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := 6;
        RETURN b.PowerOf(2)                     (* 36 *)
    END Run;
END M_Method_With_LocalVar.
