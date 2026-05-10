MODULE M_Method_On_RecordField;
    TYPE
        InnerDesc = EXTENSIBLE RECORD n: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        Outer     = RECORD inner: Inner END;

    PROCEDURE (i: Inner) Triple* (): INTEGER, NEW;
    BEGIN RETURN i.n * 3 END Triple;

    PROCEDURE Run* (): INTEGER;
        VAR o: Outer;
    BEGIN
        NEW(o.inner);
        o.inner.n := 7;
        RETURN o.inner.Triple()
    END Run;
END M_Method_On_RecordField.
