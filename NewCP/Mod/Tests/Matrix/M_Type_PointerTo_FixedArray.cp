MODULE M_Type_PointerTo_FixedArray;
    TYPE Buf = POINTER TO ARRAY 8 OF INTEGER;

    PROCEDURE Run* (): INTEGER;
        VAR p: Buf;
    BEGIN
        NEW(p);
        p[3] := 77;
        RETURN p[3]
    END Run;
END M_Type_PointerTo_FixedArray.
