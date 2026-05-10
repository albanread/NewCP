MODULE M_AnyPtr_TypeGuard;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;

    PROCEDURE Run* (): INTEGER;
        VAR
            b: Box;
            any: ANYPTR;
    BEGIN
        NEW(b);
        b.value := 73;
        any := b;
        RETURN any(Box).value
    END Run;
END M_AnyPtr_TypeGuard.
