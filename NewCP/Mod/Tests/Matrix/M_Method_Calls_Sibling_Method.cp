MODULE M_Method_Calls_Sibling_Method;
    TYPE
        BoxDesc = EXTENSIBLE RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) Get* (): INTEGER, NEW;
    BEGIN RETURN b.value END Get;

    PROCEDURE (b: Box) DoubleViaGet* (): INTEGER, NEW;
    BEGIN RETURN b.Get() * 2 END DoubleViaGet;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.value := 50;
        RETURN b.DoubleViaGet()         (* 100 *)
    END Run;
END M_Method_Calls_Sibling_Method.
