MODULE M_Expr_HexBit_HighBit;
    PROCEDURE Run* (): INTEGER;
        VAR n: INTEGER;
    BEGIN
        n := 7FFFFFFFH;        (* INT32 max as a hex literal *)
        RETURN n
    END Run;
END M_Expr_HexBit_HighBit.
