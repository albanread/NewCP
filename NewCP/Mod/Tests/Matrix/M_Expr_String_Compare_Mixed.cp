MODULE M_Expr_String_Compare_Mixed;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: ARRAY 8 OF CHAR; score: INTEGER;
    BEGIN
        a := "abc";
        b := "abd";
        score := 0;
        IF a < b  THEN score := score + 1   END;
        IF a <= b THEN score := score + 10  END;
        IF b > a  THEN score := score + 100 END;
        RETURN score
    END Run;
END M_Expr_String_Compare_Mixed.
