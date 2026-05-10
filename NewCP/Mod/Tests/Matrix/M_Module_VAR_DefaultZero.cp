MODULE M_Module_VAR_DefaultZero;
    TYPE
        BoxDesc = RECORD x: INTEGER END;
        Box     = POINTER TO BoxDesc;

    VAR
        n: INTEGER;
        flag: BOOLEAN;
        ptr: Box;

    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF n = 0      THEN score := score + 1   END;
        IF ~flag      THEN score := score + 10  END;
        IF ptr = NIL  THEN score := score + 100 END;
        RETURN score
    END Run;
END M_Module_VAR_DefaultZero.
