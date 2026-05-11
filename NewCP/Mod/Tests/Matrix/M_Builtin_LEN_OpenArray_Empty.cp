MODULE M_Builtin_LEN_OpenArray_Empty;
    TYPE Vec = POINTER TO ARRAY OF INTEGER;

    PROCEDURE Run* (): INTEGER;
        VAR p: Vec;
    BEGIN
        NEW(p, 0);
        RETURN LEN(p^)
    END Run;
END M_Builtin_LEN_OpenArray_Empty.
