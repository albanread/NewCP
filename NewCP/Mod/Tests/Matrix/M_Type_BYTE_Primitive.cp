MODULE M_Type_BYTE_Primitive;
    PROCEDURE Run* (): INTEGER;
        VAR b: BYTE; n: INTEGER;
    BEGIN
        b := SHORT(SHORT(SHORT(100)));
        n := b * 2;
        RETURN n
    END Run;
END M_Type_BYTE_Primitive.
