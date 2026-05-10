MODULE M_Expr_ENTIER_NegativeReal;
    PROCEDURE Run* (): LONGINT;
        VAR a, b, c: LONGINT;
    BEGIN
        a := ENTIER( 2.7);     (*  2 *)
        b := ENTIER(-2.3);     (* -3 *)
        c := ENTIER(-2.7);     (* -3 *)
        (* a*100 + (b+10)*10 + (c+10) + 3
           = 2*100 + 7*10 + 7 + 3 = 280 *)
        RETURN a * 100 + (b + 10) * 10 + (c + 10) + 3
    END Run;
END M_Expr_ENTIER_NegativeReal.
