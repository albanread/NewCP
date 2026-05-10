MODULE M_Expr_MixedWidth_Arithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR s: SHORTINT; n: INTEGER; l: LONGINT;
    BEGIN
        s := 34;
        n := 200;
        l := 1000;
        RETURN l + n + s          (* 1000 + 200 + 34 = 1234 *)
    END Run;
END M_Expr_MixedWidth_Arithmetic.
