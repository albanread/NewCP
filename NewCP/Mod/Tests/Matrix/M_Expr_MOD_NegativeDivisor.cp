MODULE M_Expr_MOD_NegativeDivisor;
    PROCEDURE Run* (): INTEGER;
        VAR a, b, score: INTEGER;
    BEGIN
        a :=    7  MOD (-3);     (* CP: -2  (7 = -3*-3 + -2 = 9 - 2) *)
        b := (-7) MOD (-3);      (* CP: -1  (-7 = -3*3 + -1 = -6 - 1) *)
        score := 0;
        IF a = -2 THEN score := score + 1  END;
        IF b = -1 THEN score := score + 10 END;
        RETURN score             (* 11 *)
    END Run;
END M_Expr_MOD_NegativeDivisor.
