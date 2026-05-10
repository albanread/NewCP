MODULE M_Expr_Hex_Literals;
    PROCEDURE Run* (): INTEGER;
        VAR x: INTEGER;
    BEGIN
        x := 0FFH + 100H;       (* 255 + 256 = 511 *)
        RETURN x
    END Run;
END M_Expr_Hex_Literals.
