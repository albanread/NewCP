MODULE M_Record_Field_DefaultZero;
    TYPE
        InnerDesc = RECORD x: INTEGER END;
        Inner     = POINTER TO InnerDesc;
        ItemDesc  = RECORD
            n*: INTEGER;
            flag*: BOOLEAN;
            next*: Inner
        END;
        Item      = POINTER TO ItemDesc;

    PROCEDURE Run* (): INTEGER;
        VAR p: Item; score: INTEGER;
    BEGIN
        NEW(p);
        score := 0;
        IF p.n = 0      THEN score := score + 1    END;
        IF ~p.flag      THEN score := score + 10   END;
        IF p.next = NIL THEN score := score + 100  END;
        score := score + 1000;
        RETURN score
    END Run;
END M_Record_Field_DefaultZero.
