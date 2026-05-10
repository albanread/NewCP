MODULE M_Builtin_ABS_ODD_MIN_MAX;
    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF ABS(-7) = 7 THEN score := score + 1 END;
        IF ODD(7)      THEN score := score + 10 END;
        IF ~ODD(8)     THEN score := score + 100 END;
        IF MAX(INTEGER) > 0 THEN score := score + 1000 END;
        RETURN score
    END Run;
END M_Builtin_ABS_ODD_MIN_MAX.
