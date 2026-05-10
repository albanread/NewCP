MODULE M_Method_ReturnsBoolean;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) IsPositive* (): BOOLEAN, NEW;
    BEGIN RETURN b.v > 0 END IsPositive;

    PROCEDURE Run* (): INTEGER;
        VAR b: Box;
    BEGIN
        NEW(b);
        b.v := 42;
        IF b.IsPositive() THEN RETURN b.v ELSE RETURN -1 END
    END Run;
END M_Method_ReturnsBoolean.
