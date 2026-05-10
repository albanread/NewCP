MODULE M_Expr_SET_Equality;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: SET; score: INTEGER;
    BEGIN
        a := {1, 3, 5, 7};
        b := {3, 5} + {1, 7};
        score := 0;
        IF a = b           THEN score := score + 1    END;
        IF {3, 5} <= a     THEN score := score + 10   END;
        IF a >= {3, 5}     THEN score := score + 100  END;
        IF ~({0} <= a)     THEN score := score + 1000 END;
        RETURN score
    END Run;
END M_Expr_SET_Equality.
