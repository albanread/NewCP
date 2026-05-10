MODULE M_Expr_REAL_Relational;
    PROCEDURE Run* (): INTEGER;
        VAR x, y: REAL; score: INTEGER;
    BEGIN
        x := 1.5; y := 2.0;
        score := 0;
        IF x <  y  THEN score := score + 1     END;
        IF y >  x  THEN score := score + 10    END;
        IF x <= x  THEN score := score + 100   END;
        IF x =  1.5 THEN score := score + 1000 END;
        RETURN score
    END Run;
END M_Expr_REAL_Relational.
