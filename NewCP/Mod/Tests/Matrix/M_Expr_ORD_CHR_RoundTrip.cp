MODULE M_Expr_ORD_CHR_RoundTrip;
    PROCEDURE Run* (): INTEGER;
        VAR a, b: INTEGER;
    BEGIN
        a := ORD(CHR(65));         (* 65 = "A" *)
        b := ORD(CHR(192));        (* 192 — out of ASCII, still valid CHAR *)
        RETURN a + b               (* 65 + 192 = 257 *)
    END Run;
END M_Expr_ORD_CHR_RoundTrip.
