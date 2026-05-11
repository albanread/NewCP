MODULE M_Expr_DIV_MOD_Identity;
    PROCEDURE Holds (a, b: INTEGER): BOOLEAN;
    BEGIN RETURN (a DIV b) * b + (a MOD b) = a END Holds;

    PROCEDURE Run* (): INTEGER;
        VAR score: INTEGER;
    BEGIN
        score := 0;
        IF Holds( 7,  3) THEN score := score + 1    END;
        IF Holds(-7,  3) THEN score := score + 10   END;
        IF Holds( 7, -3) THEN score := score + 100  END;
        IF Holds(-7, -3) THEN score := score + 1000 END;
        RETURN score                                   (* 1111 *)
    END Run;
END M_Expr_DIV_MOD_Identity.
