MODULE M_Expr_CHAR_Hex_Literals;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: CHAR;
    BEGIN
        a := 41X;       (* 0x41 = 65 = "A" *)
        b := "A";
        IF a = b THEN RETURN 1 ELSE RETURN 0 END
    END Run;
END M_Expr_CHAR_Hex_Literals.
