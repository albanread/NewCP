MODULE M_Expr_LONGINT_Relational;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: LONGINT; score: INTEGER;
    BEGIN
        a := 10000000000;
        b := 20000000000;
        score := 0;
        IF a < b           THEN score := score + 1   END;
        IF b > a           THEN score := score + 10  END;
        IF a + a = b       THEN score := score + 100 END;
        RETURN score
    END Run;
END M_Expr_LONGINT_Relational.
