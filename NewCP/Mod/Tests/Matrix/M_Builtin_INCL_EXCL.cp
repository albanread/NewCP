MODULE M_Builtin_INCL_EXCL;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET; score: INTEGER;
    BEGIN
        s := {};
        INCL(s, 3);
        INCL(s, 7);
        INCL(s, 11);
        EXCL(s, 7);
        score := 0;
        IF  3 IN s THEN score := score +   1 END;
        IF  7 IN s THEN score := score + 1000 END;   (* must not fire *)
        IF 11 IN s THEN score := score + 10 END;
        IF ~(7 IN s) THEN score := score + 200 END;
        RETURN score
    END Run;
END M_Builtin_INCL_EXCL.
