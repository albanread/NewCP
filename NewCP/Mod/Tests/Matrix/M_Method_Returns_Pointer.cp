MODULE M_Method_Returns_Pointer;
    TYPE
        BoxDesc = EXTENSIBLE RECORD v: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE (b: Box) WithValue* (n: INTEGER): Box, NEW;
    BEGIN
        b.v := n;
        RETURN b
    END WithValue;

    PROCEDURE Run* (): INTEGER;
        VAR b, other: Box;
    BEGIN
        NEW(b);
        other := b.WithValue(35);
        IF other = b THEN
            RETURN other.v
        ELSE
            RETURN -1
        END
    END Run;
END M_Method_Returns_Pointer.
