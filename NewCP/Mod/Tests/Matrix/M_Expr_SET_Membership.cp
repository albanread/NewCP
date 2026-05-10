MODULE M_Expr_SET_Membership;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET; score: INTEGER;
    BEGIN
        s := {1, 3, 5};
        score := 0;
        IF 3 IN s  THEN score := score + 1   END;
        IF 4 IN s  THEN score := score + 100 END;   (* must NOT fire *)
        IF 5 IN s  THEN score := score + 10  END;
        RETURN score
    END Run;
END M_Expr_SET_Membership.
