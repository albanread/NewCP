MODULE M_Type_EmptyRecord;
    TYPE
        VoidDesc = RECORD END;
        Void     = POINTER TO VoidDesc;

    PROCEDURE Run* (): INTEGER;
        VAR p: Void;
    BEGIN
        NEW(p);
        IF p # NIL THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Type_EmptyRecord.
