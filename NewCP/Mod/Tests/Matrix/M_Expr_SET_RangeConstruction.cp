MODULE M_Expr_SET_RangeConstruction;
    PROCEDURE Run* (): INTEGER;
        VAR s: SET; score: INTEGER;
    BEGIN
        s := {3..7};                 (* bits 3,4,5,6,7 *)
        score := 0;
        IF 3 IN s THEN score := score + 1   END;
        IF 5 IN s THEN score := score + 10  END;
        IF 7 IN s THEN score := score + 100 END;
        IF 8 IN s THEN score := score + 1000 END;   (* must not fire *)
        IF 2 IN s THEN score := score + 10000 END;  (* must not fire *)
        RETURN score + 137                          (* 111 + 137 = 248 *)
    END Run;
END M_Expr_SET_RangeConstruction.
