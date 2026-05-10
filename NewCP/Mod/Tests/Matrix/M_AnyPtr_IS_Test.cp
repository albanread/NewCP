MODULE M_AnyPtr_IS_Test;
    TYPE
        BoxDesc = RECORD value: INTEGER END;
        Box     = POINTER TO BoxDesc;
        BagDesc = RECORD count: INTEGER END;
        Bag     = POINTER TO BagDesc;

    PROCEDURE Run* (): INTEGER;
        VAR
            b: Box;
            any: ANYPTR;
            score: INTEGER;
    BEGIN
        NEW(b);
        any := b;
        score := 0;
        IF any IS Box THEN score := score + 100 END;
        IF any IS Bag THEN score := score + 1000 END;
        score := score + 10;
        RETURN score
    END Run;
END M_AnyPtr_IS_Test.
