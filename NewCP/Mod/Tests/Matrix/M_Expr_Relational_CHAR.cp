MODULE M_Expr_Relational_CHAR;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, c: CHAR; score: INTEGER;
    BEGIN
        a := "A"; b := "B"; c := "A";
        score := 0;
        IF a <  b THEN score := score + 1     END;
        IF a <= c THEN score := score + 10    END;
        IF b >  a THEN score := score + 100   END;
        IF b >= a THEN score := score + 1000  END;
        IF a =  c THEN score := score + 10000 END;
        RETURN score
    END Run;
END M_Expr_Relational_CHAR.
