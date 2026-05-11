MODULE M_Module_VAR_Record_DefaultZero;
    VAR r: RECORD a, b, c: INTEGER END;

    PROCEDURE Run* (): INTEGER;
    BEGIN
        RETURN r.a + r.b + r.c
    END Run;
END M_Module_VAR_Record_DefaultZero.
