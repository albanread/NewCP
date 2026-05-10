MODULE M_Type_RecordWith_Three_Field_Types;
    TYPE Mixed = RECORD
        n: INTEGER;
        b: BOOLEAN;
        r: REAL;
        c: CHAR
    END;

    PROCEDURE Run* (): INTEGER;
        VAR m: Mixed; score: INTEGER;
    BEGIN
        m.n := 1000;
        m.b := TRUE;
        m.r := 1.5;
        m.c := "X";
        score := 0;
        IF m.n = 1000 THEN score := score + 1000 END;
        IF m.b THEN score := score + 20 END;
        IF ENTIER(m.r * 2.0) = 3 THEN score := score + 3 END;
        IF m.c = "X" THEN score := score + 0 END;
        RETURN score                          (* 1000 + 20 + 3 = 1023 *)
    END Run;
END M_Type_RecordWith_Three_Field_Types.
