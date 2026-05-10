MODULE M_Type_Record_With_BOOLEAN_Field;
    TYPE Pair = RECORD flag: BOOLEAN; value: INTEGER END;

    PROCEDURE Run* (): INTEGER;
        VAR p: Pair;
    BEGIN
        p.flag := TRUE;
        p.value := 100;
        IF p.flag THEN RETURN p.value ELSE RETURN 0 END
    END Run;
END M_Type_Record_With_BOOLEAN_Field.
