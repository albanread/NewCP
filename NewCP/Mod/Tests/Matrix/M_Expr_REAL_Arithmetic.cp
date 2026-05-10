MODULE M_Expr_REAL_Arithmetic;
    PROCEDURE Run* (): LONGINT;
        VAR x, y, r: REAL;
    BEGIN
        x := 3.5;
        y := 2.0;
        r := (x + y) * 4.0 - 1.0;    (* (5.5)*4 - 1 = 21.0 *)
        RETURN ENTIER(r)
    END Run;
END M_Expr_REAL_Arithmetic.
