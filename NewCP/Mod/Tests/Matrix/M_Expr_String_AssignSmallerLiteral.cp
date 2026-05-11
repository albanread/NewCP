MODULE M_Expr_String_AssignSmallerLiteral;
    PROCEDURE Run* (): INTEGER;
        VAR arr: ARRAY 8 OF CHAR; score: INTEGER;
    BEGIN
        arr := "hi";
        score := 0;
        IF arr[0] = "h" THEN score := score + 1   END;
        IF arr[1] = "i" THEN score := score + 10  END;
        IF arr[2] = 0X  THEN score := score + 100 END;
        RETURN score                              (* 111 *)
    END Run;
END M_Expr_String_AssignSmallerLiteral.
