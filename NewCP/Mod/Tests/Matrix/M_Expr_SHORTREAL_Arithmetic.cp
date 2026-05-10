MODULE M_Expr_SHORTREAL_Arithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR x, y: SHORTREAL;
    BEGIN
        x := SHORT(3.0);
        y := SHORT(2.5);
        RETURN ENTIER(x * y * 2.4)      (* 3.0*2.5*2.4 = 18.0 → 18 *)
    END Run;
END M_Expr_SHORTREAL_Arithmetic.
