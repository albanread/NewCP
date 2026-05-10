MODULE M_Param_IN_Pointer_Deref;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Peek (IN b: Box): INTEGER;
    BEGIN RETURN b.value END Peek;

    PROCEDURE Run* (): INTEGER;
        VAR p: Box;
    BEGIN
        NEW(p);
        p.value := 42;
        RETURN Peek(p)
    END Run;
END M_Param_IN_Pointer_Deref.
