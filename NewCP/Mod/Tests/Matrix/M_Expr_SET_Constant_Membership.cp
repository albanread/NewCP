MODULE M_Expr_SET_Constant_Membership;
    CONST evens = {0, 2, 4, 6, 8};

    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF ~(3 IN evens) THEN score := score + 1    END;
        IF   4 IN evens  THEN score := score + 10   END;
        IF ~(5 IN evens) THEN score := score + 100  END;
        IF   8 IN evens  THEN score := score + 1000 END;
        RETURN score                                  (* 1111 *)
    END Run;
END M_Expr_SET_Constant_Membership.
