MODULE M_Builtin_MIN_MAX_OfType;
    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF MIN(INTEGER) < 0      THEN score := score + 1 END;
        IF MAX(INTEGER) > 0      THEN score := score + 2 END;
        IF MIN(INTEGER) < MAX(INTEGER) THEN score := score + 1 END;
        RETURN score                          (* 1 + 2 + 1 = 4 *)
    END Run;
END M_Builtin_MIN_MAX_OfType.
