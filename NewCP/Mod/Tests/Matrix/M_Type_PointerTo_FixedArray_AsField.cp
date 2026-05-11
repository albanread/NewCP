MODULE M_Type_PointerTo_FixedArray_AsField;
    TYPE
        Buf  = POINTER TO ARRAY 4 OF INTEGER;
        Wrap = RECORD b: Buf END;

    PROCEDURE Run* (): INTEGER;
        VAR w: Wrap;
    BEGIN
        NEW(w.b);
        w.b[2] := 55;
        RETURN w.b[2]
    END Run;
END M_Type_PointerTo_FixedArray_AsField.
